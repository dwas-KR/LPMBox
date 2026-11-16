//! LTBox-inspired partition wizard for the MTK SPFlashTool backend.
//! Unlike LTBox Qualcomm EDL/GPT, the selection table comes from the ROM scatter.
use super::*;
use std::sync::{Arc,Mutex};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub(super) enum PartitionMode { Dump, Flash }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum PartitionSort { Region, Name, Start, Size }
#[derive(Debug,Clone)]
pub(super) enum PartitionWizardMsg {
    Open(PartitionMode),
    Close,
    ChooseFirmware,
    Scan,
    Filter(String),
    Toggle(usize),
    ToggleAll,
    Sort(PartitionSort),
    ChooseImage(usize),
    Back,
    Next,
    ChooseDumpFolder,
    ConfirmFlash,
    Tick,
    SaveLog,
    Done(std::result::Result<String,String>),
    Reset,
}
#[derive(Debug,Clone)]
struct PartitionProgress {
    percent:f32,
    stage:String,
}
impl Default for PartitionProgress{
    fn default()->Self{
        Self{percent:0.0,stage:"ADB 연결 확인".into()}
    }
}
fn report_partition_progress(progress:&Arc<Mutex<PartitionProgress>>,
    percent:f32,stage:impl Into<String>){
    if let Ok(mut state)=progress.lock(){
        state.percent=percent.clamp(0.0,100.0);
        state.stage=stage.into();
    }
}
#[derive(Debug,Clone)]
struct PartitionEntryRow {
    meta: lpmbox_spft::ScatterPartition,
    selected: bool,
    flash_image: Option<PathBuf>,
}
pub(super) struct PartitionWizard {
    pub(super) mode: PartitionMode,
    step: u8,
    firmware: Option<PathBuf>,
    rows: Vec<PartitionEntryRow>,
    filter: String,
    sort: PartitionSort,
    descending: bool,
    status: String,
    folder_error: Option<String>,
    lines: Arc<Mutex<Vec<String>>>,
    view_lines: Vec<String>,
    progress: Arc<Mutex<PartitionProgress>>,
    view_progress: PartitionProgress,
    pub(super) running: bool,
    pub(super) result: Option<bool>,
}
impl PartitionWizard {
    fn new(mode:PartitionMode,firmware:Option<PathBuf>)->Self{
        Self{
            mode,step:0,firmware,rows:Vec::new(),filter:String::new(),
            sort:PartitionSort::Name,descending:false,
            status:"펌웨어 image 폴더를 선택하고 '파티션 검색'을 누르세요.".into(),
            folder_error:None,
            lines:Arc::new(Mutex::new(Vec::new())),view_lines:Vec::new(),
            progress:Arc::new(Mutex::new(PartitionProgress::default())),
            view_progress:PartitionProgress::default(),
            running:false,result:None,
        }
    }
    pub(super) fn name(&self)->&'static str{
        match self.mode { PartitionMode::Dump=>"파티션 덤프", PartitionMode::Flash=>"파티션 플래싱" }
    }
    fn visible(&self)->Vec<usize>{
        let needle=self.filter.trim().to_ascii_lowercase();
        let mut rows:Vec<usize>=self.rows.iter().enumerate()
            .filter(|(_,r)|needle.is_empty()
                || r.meta.name.to_ascii_lowercase().contains(&needle))
            .map(|(i,_)|i).collect();
        rows.sort_by(|a,b|{
            let l=&self.rows[*a].meta;
            let r=&self.rows[*b].meta;
            let order=match self.sort {
                PartitionSort::Region=>l.region.cmp(&r.region),
                PartitionSort::Name=>l.name.cmp(&r.name),
                PartitionSort::Start=>
                    parse_scatter_u64(&l.start_hex).cmp(&parse_scatter_u64(&r.start_hex)),
                PartitionSort::Size=>l.size_bytes.cmp(&r.size_bytes),
            };
            let order=if self.descending{order.reverse()}else{order};
            order.then(l.name.cmp(&r.name))
        });
        rows
    }
    fn count_size(&self)->(usize,u64){
        self.rows.iter().filter(|r|r.selected).fold((0,0),
            |(n,size),r|(n+1,size.saturating_add(r.meta.size_bytes)))
    }
    fn fully_assigned(&self)->bool{
        self.rows.iter().filter(|r|r.selected).all(|r|
            r.flash_image.as_ref().is_some_and(|p|p.is_file()))
    }
    fn selections(&self)->Vec<lpmbox_spft::PartitionSelection>{
        self.rows.iter().filter(|r|r.selected).map(|r|
            lpmbox_spft::PartitionSelection{
                name:r.meta.name.clone(),image:r.flash_image.clone()
            }).collect()
    }
}
fn parse_scatter_u64(raw:&str)->u64{
    u64::from_str_radix(raw.trim().trim_start_matches("0x").trim_start_matches("0X"),16)
        .unwrap_or_default()
}
fn display_size(bytes:u64)->String{
    if bytes >= 1024*1024*1024 {
        format!("{:.2} GB",bytes as f64/(1024f64*1024f64*1024f64))
    } else if bytes>=1024*1024 {
        format!("{:.2} MB",bytes as f64/(1024f64*1024f64))
    } else {
        format!("{:.2} KB",bytes as f64/1024f64)
    }
}
fn sanitized_partition_log(line:&str)->Option<String>{
    // Existing ROM preflight can emit hidden spinner control frames.
    // They belong to the ROM UI renderer, not to a partition log or TXT export.
    if line.contains("LPMBOX_SPINNER") {
        return None;
    }
    let cleaned=line
        .trim_matches(|c:char|c.is_control() && c!='\n' && c!='\t')
        .trim()
        .to_string();
    if cleaned.is_empty(){None}else{Some(cleaned)}
}
fn add_partition_log(logs:&Arc<Mutex<Vec<String>>>,text:impl Into<String>){
    let incoming=text.into();
    let Some(cleaned)=sanitized_partition_log(&incoming) else{return;};
    if let Ok(mut guard)=logs.lock(){
        guard.push(cleaned);
        if guard.len()>1500{
            guard.drain(0..200);
        }
    }
}
fn run_partition_job(
    firmware:PathBuf,
    selections:Vec<lpmbox_spft::PartitionSelection>,
    destination:Option<PathBuf>,
    log:Arc<Mutex<Vec<String>>>,
    progress:Arc<Mutex<PartitionProgress>>,
)->std::result::Result<String,String>{
    let writing=destination.is_none();
    add_partition_log(&log,format!(
        "[1/4] MediaTek ADB 사전 점검 / 선택 파티션 {}개",selections.len()));
    let model=lpmbox_device::verify_partition_mtk_device(|line|
        add_partition_log(&log,line)).map_err(|err|err.to_string())?;
    add_partition_log(&log,format!("[기기] {model}; 실기기 GPT 조회는 지원되지 않습니다."));
    report_partition_progress(&progress,0.0,"SPFlashToolV6 수신 준비");
    add_partition_log(&log,"[2/4] SPFlashToolV6 USB 수신 대기 사전 실행");
    let prearmed=lpmbox_spft::PrearmedPartitionBatch::start(
        &firmware,&selections,destination.as_deref()
    ).map_err(|err|err.to_string())?;

    // Do ADB authorization and serial discovery before starting the short
    // PreLoader watch. The USB scan and reboot must overlap immediately.
    let serial=match lpmbox_device::prepare_partition_preloader_serial(|line|
        add_partition_log(&log,line)){
        Ok(serial)=>serial,
        Err(err)=>{
            prearmed.cancel();
            return Err(format!("ADB 사전 검사 실패: {err}"));
        }
    };

    // Always arm the original ROM installation PreLoader watcher BEFORE reboot.
    // SPFT is already waiting, so the short-lived USB port cannot be missed
    // merely because the tool started after the port had appeared.
    let watch=lpmbox_device::PreloaderWatch::start(30);
    if !watch.is_armed(){
        prearmed.cancel();
        return Err("PreLoader 감시 초기화 실패: adb reboot edl 명령을 실행하지 않았습니다.".into());
    }
    report_partition_progress(&progress,0.0,"PreLoader 감지 및 ADB 재부팅");
    add_partition_log(&log,"[3/4] Windows COM PreLoader 감시 시작 (ROM 설치와 동일)");
    // Both the SPFT child and the existing ROM PreLoaderWatch are already running.
    // Match the verified proinfo backup flow: fail closed if the expected
    // MediaTek COM arrival is not detected, instead of leaving SPFT waiting
    // for its entire readback timeout and calling it a success.
    if let Err(err)=lpmbox_device::request_partition_preloader_edl_for_serial(&serial,|line|
        add_partition_log(&log,line)){
        prearmed.cancel();
        return Err(format!("adb reboot edl 실패: {err}"));
    }
    let detected=watch.wait();
    if !detected.detected {
        prearmed.cancel();
        let detail=detected.method.unwrap_or_else(||"활성 PreLoader COM 미발견".into());
        add_partition_log(&log,format!(
            "[PreLoader] 실기기 PreLoader를 30초 안에 감지하지 못했습니다: {detail}"
        ));
        report_partition_progress(&progress,0.0,"PreLoader 실패 · 안전한 재부팅 시도");
        let sent=lpmbox_device::recover_partition_after_failure(&serial,|message|
            add_partition_log(&log,message));
        return Err(format!("PreLoader 포트 감지 실패: {detail} / {}",
            if sent{"재부팅 요청 전송"}else{"수동 복구 안내를 확인하세요"}));
    }
    add_partition_log(&log,format!("[PreLoader] 실기기 감지: {} / {}",
        detected.display_name.unwrap_or_else(||"COM 포트".into()),
        detected.method.unwrap_or_else(||"Windows SetupAPI".into())
    ));
    report_partition_progress(&progress,0.0,
        if writing{"파티션 플래싱 · SPFT 전송"}else{"파티션 덤프 · SPFT 읽기"});
    add_partition_log(&log,"[4/4] SPFlashToolV6 작업 수행 및 실제 파일 결과 확인");
    let completed=prearmed.finish(|event|{
        match event{
            ProinfoLiveEvent::Log(line)=>add_partition_log(&log,line),
            ProinfoLiveEvent::Progress(p)=>{
                // The exact SPFT stage percent is shown, not an invented
                // overall percentage across several selected partitions.
                report_partition_progress(&progress,p.percent as f32,
                    format!("SPFT 실제 진행 · {}",p.stage));
                add_partition_log(&log,format!("[진행] {} / {}%",p.stage,p.percent));
            },
            ProinfoLiveEvent::Error(e)=>add_partition_log(&log,format!("[오류] {e}")),
            ProinfoLiveEvent::Finished(_)=>{},
        }
    });
    let finished=match completed{
        Ok(files)=>files,
        Err(err)=>{
            let detail=err.to_string();
            add_partition_log(&log,format!("[SPFT 오류] {detail}"));
            if detail.to_ascii_lowercase().contains("load-package")
                || detail.to_ascii_lowercase().contains("da mode") {
                add_partition_log(&log,
                    "[점검] DA/Load-Package가 연결된 모델·펌웨어에 호환되는지 확인하세요. 해당 오류는 PreLoader 감지 실패가 아니며 임의의 DA 교체나 인증 우회로 해결할 수 있다고 보장할 수 없습니다.");
            }
            report_partition_progress(&progress,0.0,"SPFT 작업 실패 · 안전한 재부팅 시도");
            let reboot_sent=lpmbox_device::recover_partition_after_failure(&serial,|message|
                add_partition_log(&log,message));
            report_partition_progress(&progress,0.0,
                if reboot_sent{"재부팅 요청 완료 · 정상 부팅 확인 필요"}
                else{"자동 재부팅 불가 · 전원 버튼으로 수동 복구 필요"});
            return Err(format!("{detail} / {}",
                if reboot_sent{"재부팅 명령 전송됨 (정상 부팅 미확인)"}
                else{"자동 재부팅 불가, 복구 안내 확인"}));
        }
    };
    report_partition_progress(&progress,100.0,"작업 완료");
    if writing{
        add_partition_log(&log,"[완료] SPFT가 선택한 이미지 다운로드를 종료했습니다.");
        Ok(format!(
            "파티션 플래싱 명령 완료 ({}개). 정상 부팅·서명·AVB 상태는 실기기에서 확인해야 합니다.",
            selections.len()
        ))
    }else{
        for (name,path,length) in &finished{
            add_partition_log(&log,format!("[덤프 완료] {name}: {} / {}",
                path.display(),display_size(*length)));
        }
        Ok(format!(
            "선택한 파티션 {}개 덤프 완료. 저장 폴더: {}",
            finished.len(),destination.unwrap_or_default().display()
        ))
    }
}

impl App {
    pub(super) fn update_partition_wizard(&mut self,msg:PartitionWizardMsg)->Task<Message>{
        match msg{
            PartitionWizardMsg::Open(mode)=>{
                // The partition tools remain implemented for later versions,
                // but are not selectable or invokable in this UI build.
                if !LPM_EXPERIMENTAL_UI_ENABLED || self.busy {
                    return Task::none();
                }
                self.active_nav=NavPage::Driver;
                let firmware=if mode==PartitionMode::Dump { None } else { self.image_dir.clone() };
                self.partition_wizard=Some(PartitionWizard::new(mode,firmware));
            }
            PartitionWizardMsg::Close=>{
                if self.partition_wizard.as_ref().is_some_and(|w|w.running){
                    return Task::none();
                }
                self.partition_wizard=None;
            }
            PartitionWizardMsg::ChooseFirmware=>{
                if self.busy{return Task::none();}
                if let Some(path)=rfd::FileDialog::new()
                    .set_title("MediaTek 펌웨어 image 폴더 선택").pick_folder(){
                    let resolved=resolve_image_folder_selection(&path);
                    if let Some(w)=self.partition_wizard.as_mut(){
                        match resolved{
                            Ok(p)=>{
                                w.firmware=Some(p);
                                w.rows.clear();
                                w.step=0;
                                w.status="선택한 펌웨어의 Scatter 파티션 목록을 분석 중입니다.".into();
                                w.folder_error=None;
                            }
                            Err(e)=>{
                                w.firmware=None;
                                w.rows.clear();
                                w.step=0;
                                w.folder_error=Some(e.to_string());
                                w.status=format!("펌웨어 경로 오류: {e}");
                            },
                        }
                    }
                    if self.partition_wizard.as_ref().is_some_and(|w|
                        w.mode==PartitionMode::Dump && w.firmware.is_some()) {
                        return self.update_partition_wizard(PartitionWizardMsg::Scan);
                    }
                }
            }
            PartitionWizardMsg::Scan=>{
                if self.busy{return Task::none();}
                let Some(w)=self.partition_wizard.as_mut()else{return Task::none();};
                let Some(path)=w.firmware.as_ref() else{
                    w.status="먼저 image 폴더를 선택하세요.".into();
                    return Task::none()
                };
                match lpmbox_spft::scan_firmware_partition_table(path){
                    Ok(entries)=>{
                        w.rows=entries.into_iter().map(|meta|PartitionEntryRow{
                            meta,selected:false,flash_image:None
                        }).collect();
                        w.step=1;
                        w.folder_error=None;
                        w.status=format!("Scatter XML에서 {}개 파티션을 확인했습니다.",w.rows.len());
                    }
                    Err(err)=>{
                        w.step=0;
                        w.folder_error=Some(err.to_string());
                        w.status=format!("파티션 검색 실패: {err}");
                    },
                }
            }
            PartitionWizardMsg::Filter(search)=>{
                if let Some(w)=self.partition_wizard.as_mut(){w.filter=search;}
            }
            PartitionWizardMsg::Sort(column)=>{
                if let Some(w)=self.partition_wizard.as_mut(){
                    if w.sort==column{w.descending=!w.descending}
                    else{w.sort=column;w.descending=false}
                }
            }
            PartitionWizardMsg::Toggle(idx)=>{
                if let Some(w)=self.partition_wizard.as_mut(){
                    if let Some(row)=w.rows.get_mut(idx){row.selected=!row.selected;}
                }
            }
            PartitionWizardMsg::ToggleAll=>{
                if let Some(w)=self.partition_wizard.as_mut(){
                    let shown=w.visible();
                    let all=shown.iter().all(|&i|w.rows[i].selected);
                    for i in shown{w.rows[i].selected=!all;}
                }
            }
            PartitionWizardMsg::ChooseImage(index)=>{
                if self.busy{return Task::none();}
                let Some(w)=self.partition_wizard.as_mut()else{return Task::none();};
                if let Some(file)=rfd::FileDialog::new()
                    .add_filter("Partition image",&["img","bin"])
                    .set_title("선택 파티션의 플래싱 이미지 선택").pick_file(){
                    if let Some(row)=w.rows.get_mut(index){
                        row.flash_image=Some(file);
                        row.selected=true;
                    }
                }
            }
            PartitionWizardMsg::Back=>{
                if let Some(w)=self.partition_wizard.as_mut(){
                    if w.running{return Task::none();}
                    if w.step>0{w.step-=1;}
                    else{self.partition_wizard=None;}
                }
            }
            PartitionWizardMsg::Next=>{
                if let Some(w)=self.partition_wizard.as_mut(){
                    if w.mode==PartitionMode::Flash
                        && w.count_size().0>0 && w.fully_assigned(){
                        w.step=2;
                    }
                }
            }
            PartitionWizardMsg::ChooseDumpFolder=>{
                if self.busy{return Task::none();}
                let Some(w)=self.partition_wizard.as_ref()else{return Task::none();};
                if w.count_size().0==0 || w.count_size().0>256{return Task::none();}
                if let Some(folder)=rfd::FileDialog::new()
                    .set_title("파티션 덤프 저장 폴더 선택").pick_folder(){
                    return self.run_partition_wizard_job(Some(folder));
                }
            }
            PartitionWizardMsg::ConfirmFlash=>{
                return self.run_partition_wizard_job(None);
            }
            PartitionWizardMsg::Tick=>{
                let mut new_logs=false;
                if let Some(w)=self.partition_wizard.as_mut(){
                    if let Ok(lines)=w.lines.lock(){
                        new_logs=w.view_lines.len()!=lines.len()
                            || w.view_lines.last()!=lines.last();
                        w.view_lines=lines.clone();
                    }
                    if let Ok(progress)=w.progress.lock(){
                        w.view_progress=progress.clone();
                    }
                }
                if new_logs {
                    return iced::widget::operation::snap_to_end(
                        iced::widget::Id::new("lpmbox-partition-history")
                    );
                }
            }
            PartitionWizardMsg::SaveLog=>{
                if let Some(w)=self.partition_wizard.as_ref(){
                    if let Some(file)=rfd::FileDialog::new()
                        .set_title("파티션 작업 내역 저장")
                        .set_file_name("LPMBOX_partition_log.txt")
                        .add_filter("Text",&["txt"]).save_file(){
                        let _=std::fs::write(file,w.view_lines.join("\r\n"));
                    }
                }
            }
            PartitionWizardMsg::Done(result)=>{
                self.busy=false;
                if let Some(w)=self.partition_wizard.as_mut(){
                    w.running=false;
                    w.result=Some(result.is_ok());
                    w.status=match result{
                        Ok(s)=>format!("[완료] {s}"),
                        Err(e)=>format!("[오류] {e}"),
                    };
                    if let Ok(lines)=w.lines.lock(){
                        w.view_lines=lines.clone();
                    }
                    if let Ok(progress)=w.progress.lock(){
                        w.view_progress=progress.clone();
                    }
                    w.view_lines.push(w.status.clone());
                    // The result remains available from the left "작업 내역" tab.
                    self.active_nav=NavPage::Log;
                    return iced::widget::operation::snap_to_end(
                        iced::widget::Id::new("lpmbox-partition-history")
                    );
                }
            }
            PartitionWizardMsg::Reset=>{
                if let Some(w)=self.partition_wizard.as_ref(){
                    if w.running{return Task::none();}
                    let mode=w.mode;
                    let fw=w.firmware.clone();
                    self.partition_wizard=Some(PartitionWizard::new(
                        mode,if mode==PartitionMode::Dump{None}else{fw}
                    ));
                    self.active_nav=NavPage::Driver;
                }
            }
        }
        Task::none()
    }
    fn run_partition_wizard_job(&mut self,output:Option<PathBuf>)->Task<Message>{
        if self.busy{return Task::none();}
        let Some(w)=self.partition_wizard.as_mut()else{return Task::none();};
        let Some(fw)=w.firmware.clone()else{return Task::none();};
        let selected=w.selections();
        if selected.is_empty()||selected.len()>256{return Task::none();}
        if w.mode==PartitionMode::Dump && output.is_none(){return Task::none();}
        if w.mode==PartitionMode::Flash && (!w.fully_assigned()||output.is_some()){
            return Task::none();
        }
        w.step=if w.mode==PartitionMode::Dump{2}else{3};
        w.running=true;
        w.result=None;
        w.lines=Arc::new(Mutex::new(vec![
            format!("[작업] {} / {}개 선택",w.name(),selected.len())
        ]));
        w.view_lines=Vec::new();
        w.progress=Arc::new(Mutex::new(PartitionProgress::default()));
        w.view_progress=PartitionProgress::default();
        w.status="SPFlashToolV6 및 PreLoader 감시를 준비하고 있습니다.".into();
        self.busy=true;
        let logs=Arc::clone(&w.lines);
        let progress=Arc::clone(&w.progress);
        Task::perform(async move{
            match tokio::task::spawn_blocking(move||
                run_partition_job(fw,selected,output,logs,progress)).await{
                    Ok(outcome)=>outcome,
                    Err(err)=>Err(format!("작업 스레드 오류: {err}")),
                }
        },|r|Message::PartitionWizard(PartitionWizardMsg::Done(r)))
    }
    pub(super) fn partition_wizard_panel(&self)->Element<'_,Message>{
        let Some(w)=self.partition_wizard.as_ref()else{
            return container(text("파티션 도구를 선택하세요.")).into();
        };
        if w.mode==PartitionMode::Dump {
            return match w.step {
                0=>self.partition_dump_folder_step(w),
                1=>self.partition_select_step(w),
                _=>self.partition_execute_step(w),
            };
        }
        let subtitle="MediaTek SP Flash Tool V6 · 펌웨어 Scatter 기반 (기기 GPT 스캔 아님)";
        let title=iced::widget::column![
            text(w.name()).size(22).font(lpm_bold_font()),
            shell_muted(subtitle,12),
        ].spacing(4);
        let steps=["1  펌웨어","2  파티션 선택","3  확인","4  플래싱"];
        let mut ribbon=row![].spacing(8).width(Length::Fill);
        for (idx,label) in steps.iter().enumerate(){
            let current=idx==w.step as usize;
            let label=if current{format!("● {label}")}else{format!("○ {label}")};
            ribbon=ribbon.push(container(text(label).size(12))
                .padding([8,10]).width(Length::FillPortion(1))
                .style(move |theme:&Theme|{
                    let c=shell_colors(theme);
                    iced::widget::container::Style{
                        background:Some(Background::Color(
                            if current{c.tint}else{c.panel}
                        )),
                        border:iced::Border{
                            radius:9.0.into(),width:1.0,color:c.border
                        },
                        ..Default::default()
                    }
                }));
        }
        let body:Element<'_,Message>=match w.step{
            0=>self.partition_firmware_step(w),
            1=>self.partition_select_step(w),
            2=>self.partition_flash_review_step(w),
            _=>self.partition_execute_step(w),
        };
        container(iced::widget::column![
            container(title).padding([12,20]).width(Length::Fill),
            container(ribbon).padding([0,20]).width(Length::Fill),
            body,
        ].spacing(10).width(Length::Fill).height(Length::Fill))
            .width(Length::Fill).height(Length::Fill).into()
    }
    fn partition_dump_folder_step(&self,w:&PartitionWizard)->Element<'_,Message>{
        // Mirrors rom_select_folder_card (icons, sizes, typography, padding,
        // and style) without changing the ROM workflow or its navigation.
        let folder_card_content=container(
            iced::widget::column![
                iced::widget::image(if self.uses_dark_icon_palette(){
                    self.folder_select_icon_dark_handle.clone()
                }else{
                    self.folder_select_icon_handle.clone()
                })
                    .width(Length::Fixed(90.0))
                    .height(Length::Fixed(90.0)),
                text("image 폴더를 선택해주세요.")
                    .size(13)
                    .font(lpm_font())
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
                    .wrapping(iced::widget::text::Wrapping::Word),
            ]
            .spacing(20)
            .width(Length::Shrink)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fixed(320.0))
        .height(Length::Fixed(190.0))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(lpm_nav_rom_select_card_style);

        let mut folder_button=button(folder_card_content)
            .padding(0)
            .style(lpm_nav_flat_button_style);
        if !self.busy{
            folder_button=folder_button
                .on_press(Message::PartitionWizard(PartitionWizardMsg::ChooseFirmware));
        }
        // Use the exact red ROM validation panel, centered together with
        // the identical picker tile. Never put a tiny error below the footer.
        let center_content:Element<'_,Message>=if let Some(error)=&w.folder_error{
            iced::widget::column![
                folder_button,
                self.rom_firmware_error_panel(error)
            ].spacing(14).width(Length::Fill)
                .align_x(iced::Alignment::Center).into()
        }else{
            container(folder_button).width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center).into()
        };
        let content=iced::widget::column![
            container(center_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            row![
                container(shell_action("이전".into(),
                    Message::PartitionWizard(PartitionWizardMsg::Close),
                    !self.busy,false))
                    .width(Length::Fixed(140.0)),
                Space::new().width(Length::Fill),
            ].spacing(10),
        ].height(Length::Fill).width(Length::Fill);
        container(content)
            .width(Length::Fill).height(Length::Fill)
            .padding(iced::Padding{top:24.0,right:10.0,bottom:24.0,left:10.0})
            .into()
    }
    fn partition_firmware_step(&self,w:&PartitionWizard)->Element<'_,Message>{
        let chosen=w.firmware.as_ref().map(|p|p.display().to_string())
            .unwrap_or_else(||"펌웨어 image 폴더를 선택해주세요.".into());
        let status=shell_muted(&w.status,12);
        let section=iced::widget::column![
            text("MediaTek 펌웨어 폴더 선택").size(19).font(lpm_bold_font()),
            shell_muted("MTK는 flash.xml과 Scatter XML이 포함된 image 폴더를 사용합니다.",12),
            container(text(chosen).size(12)).width(Length::Fill)
                .padding(14).style(shell_panel_style),
            shell_action("찾기 (펌웨어 image 폴더)".into(),
                Message::PartitionWizard(PartitionWizardMsg::ChooseFirmware),!self.busy,false),
            status
        ].spacing(14).width(Length::Fill);
        let buttons=row![
            shell_action("취소".into(),Message::PartitionWizard(PartitionWizardMsg::Close),
                !self.busy,false),
            Space::new().width(Length::Fill),
            shell_action("파티션 검색".into(),
                Message::PartitionWizard(PartitionWizardMsg::Scan),
                !self.busy && w.firmware.is_some(),true),
        ].spacing(12).align_y(iced::Alignment::Center);
        container(iced::widget::column![
            container(section).padding(20).width(Length::Fill).style(shell_main_style),
            Space::new().height(Length::Fill),
            container(buttons).padding([12,20]).width(Length::Fill)
        ].spacing(12).height(Length::Fill))
            .padding(12).height(Length::Fill).width(Length::Fill).into()
    }
    fn partition_select_step(&self,w:&PartitionWizard)->Element<'_,Message>{
        let rows=w.visible();
        let has_items=!rows.is_empty();
        let all_checked=has_items && rows.iter().all(|i|w.rows[*i].selected);
        let mut header=row![
            container(iced::widget::checkbox(all_checked)
                .on_toggle(|_|Message::PartitionWizard(PartitionWizardMsg::ToggleAll)))
                .width(Length::Fixed(38.0)),
            partition_sort_button("영역",PartitionSort::Region),
            partition_sort_button("레이블",PartitionSort::Name),
            partition_sort_button("시작 주소",PartitionSort::Start),
            partition_sort_button("용량",PartitionSort::Size),
        ].spacing(8).align_y(iced::Alignment::Center).padding([7,12]);
        // Keep a dedicated action column for selecting the flash image per row.
        if w.mode==PartitionMode::Flash{
            header=header.push(text("이미지 파일").size(12).width(Length::FillPortion(2)));
        }
        let mut list=iced::widget::column![].spacing(0).width(Length::Fill);
        for idx in rows{
            let r=&w.rows[idx];
            let selected=r.selected;
            let mut line=row![
                container(iced::widget::checkbox(r.selected)
                    .on_toggle(move |_|Message::PartitionWizard(PartitionWizardMsg::Toggle(idx))))
                    .width(Length::Fixed(38.0)),
                text(r.meta.region.clone()).size(11).width(Length::FillPortion(2)),
                text(r.meta.name.clone()).size(12).width(Length::FillPortion(3)),
                text(r.meta.start_hex.clone()).size(11).width(Length::FillPortion(2)),
                text(display_size(r.meta.size_bytes)).size(11).width(Length::FillPortion(2)),
            ].spacing(8).align_y(iced::Alignment::Center).padding([7,12]);
            if w.mode==PartitionMode::Flash{
                let path=r.flash_image.as_ref().map(|p|
                    p.file_name().unwrap_or_default().to_string_lossy().into_owned())
                    .unwrap_or_else(||"파일 선택".into());
                line=line.push(
                    button(text(path).size(11))
                        .on_press(Message::PartitionWizard(PartitionWizardMsg::ChooseImage(idx)))
                        .width(Length::FillPortion(2)).padding(6)
                );
            }
            let boxed=container(line).width(Length::Fill).style(move |theme:&Theme|{
                let c=shell_colors(theme);
                iced::widget::container::Style{
                    background:if selected {Some(Background::Color(c.tint))}else{None},
                    ..Default::default()
                }
            });
            list=list.push(boxed);
        }
        if !has_items{
            list=list.push(container(text("검색 조건에 일치하는 파티션이 없습니다.").size(13))
                .width(Length::Fill).padding(20));
        }
        let (count,bytes)=w.count_size();
        let can_continue=count>0 && count<=256
            && (w.mode==PartitionMode::Dump || w.fully_assigned());
        let buttons=row![
            container(shell_action("뒤로".into(),
                Message::PartitionWizard(PartitionWizardMsg::Back),!self.busy,false))
                .width(Length::Fixed(140.0)),
            Space::new().width(Length::Fill),
            container(shell_action(
                if w.mode==PartitionMode::Dump{"덤프".into()}else{"다음".into()},
                Message::PartitionWizard(if w.mode==PartitionMode::Dump{
                    PartitionWizardMsg::ChooseDumpFolder
                }else{PartitionWizardMsg::Next}),
                !self.busy && can_continue,true))
                .width(Length::Fixed(140.0)),
        ].spacing(12).align_y(iced::Alignment::Center);
        container(iced::widget::column![
            text_input("파티션 검색", &w.filter)
                .on_input(|s|Message::PartitionWizard(PartitionWizardMsg::Filter(s)))
                .padding(9).size(13),
            container(iced::widget::column![
                container(header).width(Length::Fill).style(|theme:&Theme|{
                    let c=shell_colors(theme);
                    iced::widget::container::Style{
                        background:Some(Background::Color(c.tint)),
                        ..Default::default()
                    }
                }),
                iced::widget::rule::horizontal(1),
                iced::widget::scrollable(list).height(Length::Fill),
            ].height(Length::Fill).spacing(0).width(Length::Fill))
                .height(Length::Fill).width(Length::Fill).style(shell_panel_style),
            row![
                text(format!("파티션 {}개 / 선택 {}개",w.rows.len(),count)).size(12),
                Space::new().width(Length::Fill),
                text(format!("선택한 파티션 총 용량: {}",display_size(bytes))).size(12)
            ].padding([6,2]),
            buttons
        ].spacing(8).padding(12).height(Length::Fill).width(Length::Fill))
            .height(Length::Fill).width(Length::Fill).into()
    }
    fn partition_flash_review_step(&self,w:&PartitionWizard)->Element<'_,Message>{
        let selected:Vec<&PartitionEntryRow>=w.rows.iter().filter(|r|r.selected).collect();
        let mut items=iced::widget::column![
            text("플래싱 전 최종 확인").size(18).font(lpm_bold_font()),
            shell_muted("선택한 파티션에 이미지를 실제 기록합니다. 기기별 서명·AVB 및 부팅 가능 여부는 검증되지 않았습니다.",12),
        ].spacing(10);
        for row in selected{
            items=items.push(container(iced::widget::column![
                text(row.meta.name.clone()).size(14).font(lpm_bold_font()),
                shell_muted(format!("{} · 용량 {} · 이미지 {}",
                    row.meta.region,display_size(row.meta.size_bytes),
                    row.flash_image.as_ref().map(|p|p.display().to_string())
                    .unwrap_or_else(||"미선택".into())),11),
            ].spacing(4)).padding(12).width(Length::Fill).style(shell_panel_style));
        }
        let nav=row![
            shell_action("뒤로".into(),
                Message::PartitionWizard(PartitionWizardMsg::Back),!self.busy,false),
            Space::new().width(Length::Fill),
            shell_action("플래싱 시작 (실제 기록)".into(),
                Message::PartitionWizard(PartitionWizardMsg::ConfirmFlash),!self.busy,true)
        ].spacing(12);
        container(iced::widget::column![
            container(iced::widget::scrollable(items)
                .height(Length::Fill)).height(Length::Fill).width(Length::Fill),
            nav
        ].spacing(14).padding(20).height(Length::Fill).width(Length::Fill)).into()
    }
    pub(super) fn partition_execute_step(&self,w:&PartitionWizard)->Element<'_,Message>{
        let status=if w.running{
            "작업 중 · 기기 연결을 해제하지 마세요."
        }else if w.result==Some(true){
            "완료 · 작업 로그와 결과를 확인하세요."
        }else{
            "중단/실패 · 세부 작업 로그를 확인하세요."
        };
        let stages=if w.mode==PartitionMode::Dump{
            ["ADB 기기 확인","SPFT 수신 대기","PreLoader 감시 + adb reboot edl","파티션 덤프"]
        }else{
            ["ADB 기기 확인","SPFT 수신 대기","PreLoader 감시 + adb reboot edl","파티션 플래싱"]
        };
        let lines=&w.view_lines;
        let progress=if w.running{"진행 중"}else if w.result==Some(true){"완료"}else{"실패"};
        let progress_percent=w.view_progress.percent;
        let stage_label=w.view_progress.stage.clone();
        let mut timeline=iced::widget::column![
            text(progress).size(17).font(lpm_bold_font()),
        ].spacing(16).width(Length::Fill);
        if w.mode==PartitionMode::Flash {
            timeline=timeline.push(
                iced::widget::progress_bar(0.0..=100.0,progress_percent)
            ).push(shell_muted(format!("{stage_label} · {progress_percent:.0}%"),12));
        }
        for (index,stage) in stages.iter().enumerate(){
            timeline=timeline.push(text(format!("{} {}",index+1,stage)).size(13));
        }
        let mut log_block=iced::widget::column![
            text("작업 내역").size(16).font(lpm_bold_font()),
        ].spacing(12).height(Length::Fill).width(Length::Fill);
        if w.mode==PartitionMode::Flash{
            log_block=log_block.push(
                iced::widget::progress_bar(0.0..=100.0,progress_percent)
            ).push(shell_muted(format!("{stage_label} · {}줄",lines.len()),11));
        }
        log_block=log_block.push(
            iced::widget::scrollable(
                text(lines.join("\n")).size(12)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .width(Length::Fill)
            )
            .id(iced::widget::Id::new("lpmbox-partition-history"))
            .anchor_top().height(Length::Fill)
        );
        let content:Element<'_,Message>=if self.window_size.0>920.0{
            row![
                container(timeline).padding(18).width(Length::FillPortion(1))
                    .height(Length::Fill).style(shell_panel_style),
                container(log_block).padding(18).width(Length::FillPortion(2))
                    .height(Length::Fill).style(shell_panel_style)
            ].spacing(14).height(Length::Fill).into()
        }else{
            iced::widget::column![
                container(timeline).padding(14).width(Length::Fill)
                    .style(shell_panel_style),
                container(log_block).padding(14).height(Length::Fill)
                    .width(Length::Fill).style(shell_panel_style),
            ].spacing(12).height(Length::Fill).into()
        };
        let mut footer=row![
            shell_muted(status,12),
            Space::new().width(Length::Fill),
        ].spacing(12).align_y(iced::Alignment::Center);
        if w.mode==PartitionMode::Flash{
            footer=footer.push(shell_action("로그 저장".into(),
                Message::PartitionWizard(PartitionWizardMsg::SaveLog),!w.running,false));
        }
        footer=footer.push(shell_action("처음부터".into(),
            Message::PartitionWizard(PartitionWizardMsg::Reset),!w.running,true));
        container(iced::widget::column![
            shell_muted(&w.status,12),
            container(content).height(Length::Fill).width(Length::Fill),
            footer,
        ].spacing(12).padding(16).height(Length::Fill).width(Length::Fill)).into()
    }
}
fn partition_sort_button<'a>(label:&'static str,col:PartitionSort)->Element<'a,Message>{
    button(text(label).size(11))
        .on_press(Message::PartitionWizard(PartitionWizardMsg::Sort(col)))
        .padding([4,2]).width(match col{
            PartitionSort::Region=>Length::FillPortion(2),
            PartitionSort::Name=>Length::FillPortion(3),
            PartitionSort::Start=>Length::FillPortion(2),
            PartitionSort::Size=>Length::FillPortion(2),
        }).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn label_search_does_not_match_emmc_user_region(){
        let mut w=PartitionWizard::new(PartitionMode::Dump,None);
        w.rows=vec![
            PartitionEntryRow{meta:lpmbox_spft::ScatterPartition{
                name:"boot_a".into(),region:"EMMC_USER".into(),
                start_hex:"0x0".into(),size_bytes:1024
            },selected:false,flash_image:None},
            PartitionEntryRow{meta:lpmbox_spft::ScatterPartition{
                name:"userdata".into(),region:"EMMC_USER".into(),
                start_hex:"0x1".into(),size_bytes:2048
            },selected:false,flash_image:None},
        ];
        w.filter="user".into();
        let results=w.visible();
        assert_eq!(results.len(),1);
        assert_eq!(w.rows[results[0]].meta.name,"userdata");
    }
    #[test]
    fn spinner_frames_are_not_user_visible_logs(){
        assert!(sanitized_partition_log("\u{001e}LPMBOX_SPINNER\u{001f}adb_usb|ADB|USB").is_none());
        assert_eq!(sanitized_partition_log("[ADB] USB ADB 연결 완료"),
            Some("[ADB] USB ADB 연결 완료".into()));
    }
    #[test]
    fn actual_spft_progress_clamps_percent(){
        let state=Arc::new(Mutex::new(PartitionProgress::default()));
        report_partition_progress(&state,82.0,"SPFT 실제 진행 · userdata");
        assert_eq!(state.lock().unwrap().percent,82.0);
        report_partition_progress(&state,120.0,"done");
        assert_eq!(state.lock().unwrap().percent,100.0);
    }

    #[test]
    fn rom_tab_click_resets_image_folder_to_picker(){
        let (mut app,_startup)=App::new();
        app.image_dir=Some(PathBuf::from(r"G:\test\image"));
        app.rom_show_routine_select=true;
        app.rom_firmware_error=Some("invalid firmware".into());
        app.runtime_started_at=Some(Instant::now());
        let _=app.update(Message::SelectNav(NavPage::Rom));
        assert_eq!(app.active_nav,NavPage::Rom);
        assert!(app.image_dir.is_none());
        assert!(app.rom_firmware_error.is_none());
        assert!(!app.rom_show_routine_select);
        assert!(app.runtime_started_at.is_none());
        assert!(app.partition_wizard.is_none());
    }
    #[test]
    fn partition_ui_is_not_callable_while_hidden(){
        let (mut app,_startup)=App::new();
        let before=app.active_nav;
        let _=app.update(Message::PartitionWizard(
            PartitionWizardMsg::Open(PartitionMode::Dump)));
        assert!(app.partition_wizard.is_none());
        assert_eq!(app.active_nav,before);
        let _=app.update(Message::PartitionWizard(
            PartitionWizardMsg::Open(PartitionMode::Flash)));
        assert!(app.partition_wizard.is_none());
    }
    #[test]
    fn rom_navigation_clears_old_partition_result(){
        let (mut app,_startup)=App::new();
        let mut old=PartitionWizard::new(PartitionMode::Dump,None);
        old.result=Some(true);
        old.step=2;
        old.view_lines.push("old dump".into());
        app.partition_wizard=Some(old);
        app.active_nav=NavPage::Log;
        let _=app.update(Message::SelectNav(NavPage::Rom));
        assert_eq!(app.active_nav,NavPage::Rom);
        assert!(app.partition_wizard.is_none());
    }
    #[test]
    fn partition_success_and_failure_navigate_to_work_history(){
        for is_success in [true,false]{
            let (mut app,_startup)=App::new();
            app.partition_wizard=Some(PartitionWizard::new(PartitionMode::Dump,None));
            app.active_nav=NavPage::Driver;
            app.busy=true;
            let result=if is_success {Ok("saved".into())}
                else{Err("DA mode failed".into())};
            let _=app.update(Message::PartitionWizard(PartitionWizardMsg::Done(result)));
            assert!(!app.busy);
            assert_eq!(app.active_nav,NavPage::Log);
            let wizard=app.partition_wizard.as_ref().unwrap();
            assert_eq!(wizard.result,Some(is_success));
            assert!(wizard.view_lines.iter().any(|line|
                if is_success{line.contains("[완료]")}
                else{line.contains("[오류]")}));
        }
    }

    #[test]
    fn selected_count_matches_size(){
        let mut w=PartitionWizard::new(PartitionMode::Dump,None);
        w.rows=vec![PartitionEntryRow{
            meta:lpmbox_spft::ScatterPartition{
                name:"boot_a".into(),region:"EMMC_USER".into(),
                start_hex:"0x0".into(),size_bytes:1024
            },selected:true,flash_image:None
        }];
        assert_eq!(w.count_size(),(1,1024));
    }
}

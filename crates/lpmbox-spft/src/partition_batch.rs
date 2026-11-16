//! MTK SPFlashTool V6 partition batch: scatter metadata, prearmed readback / download.
//! Separate from the original ROM installation plan; device GPT is not read.
use super::*;
use super::partition_tools::{FLASH_ALLOWED, partition_name, xml_tag, set_xml_tag, find_scatter_xml};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug,Clone)]
pub struct ScatterPartition {
    pub name: String,
    pub region: String,
    pub start_hex: String,
    pub size_bytes: u64,
}
#[derive(Debug,Clone)]
pub struct PartitionSelection {
    pub name: String,
    pub image: Option<PathBuf>,
}

fn parse_hex(src: &str) -> Option<u64> {
    let value=src.trim().trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(value,16).ok()
}
fn scan_scatter(source:&str)->Result<Vec<ScatterPartition>> {
    let mut entries=Vec::new();
    let mut rest=source;
    let mut uniq=HashSet::new();
    while let Some(start)=rest.find("<partition_index ") {
        rest=&rest[start..];
        let end=rest.find("</partition_index>").ok_or_else(||
            LpmError::Spft("scatter partition_index XML 손상".into()))?
            +"</partition_index>".len();
        let block=&rest[..end];
        let name=xml_tag(block,"partition_name").unwrap_or_default();
        let region=xml_tag(block,"region").unwrap_or_default();
        let start=xml_tag(block,"physical_start_addr")
            .or_else(||xml_tag(block,"linear_start_addr"))
            .unwrap_or("0x0");
        let size=xml_tag(block,"partition_size")
            .and_then(parse_hex).unwrap_or(0);
        if !name.is_empty() && size>0 && partition_name(name).is_ok()
            && uniq.insert(name.to_string()) {
            entries.push(ScatterPartition{
                name:name.to_string(),region:region.to_string(),
                start_hex:start.to_string(),size_bytes:size
            });
        }
        rest=&rest[end..];
    }
    if entries.is_empty(){
        return Err(LpmError::Spft("scatter에서 유효한 파티션을 찾지 못했습니다.".into()));
    }
    Ok(entries)
}
pub fn scan_firmware_partition_table(image_path:&Path)->Result<Vec<ScatterPartition>> {
    let (_,image_dir)=resolve_root_and_image_dir(image_path)?;
    let scatter=find_scatter_xml(&image_dir)?;
    let source=fs::read_to_string(scatter)?;
    scan_scatter(&source)
}

fn approved_flash_partition(name:&str)->bool {
    // Never write boot chain B, NVRAM, calibration, metadata, proinfo, preloader.
    FLASH_ALLOWED.contains(&name)
}
fn validate_selection(image_dir:&Path, selected:&[PartitionSelection], writing:bool)
->Result<Vec<ScatterPartition>> {
    if selected.is_empty() || selected.len()>256{
        return Err(LpmError::Spft("1~256개의 파티션만 작업할 수 있습니다.".into()));
    }
    let table=scan_firmware_partition_table(image_dir)?;
    let mut result=Vec::new();
    let mut seen=HashSet::new();
    for selection in selected{
        partition_name(&selection.name)?;
        if !seen.insert(selection.name.as_str()){
            return Err(LpmError::Spft("중복된 파티션 선택".into()));
        }
        let entry=table.iter().find(|e|e.name==selection.name).ok_or_else(||
            LpmError::Spft(format!("scatter 파티션 목록에 {} 없음",selection.name)))?;
        if writing{
            if !approved_flash_partition(&entry.name) {
                return Err(LpmError::Spft(format!(
                    "{}은(는) 고위험·기기 고유 또는 미검증 대상이므로 플래싱이 차단됩니다.",entry.name)));
            }
            let image=selection.image.as_ref().ok_or_else(||
                LpmError::Spft(format!("{} 이미지 파일 미선택",entry.name)))?;
            let length=fs::metadata(image)?.len();
            if length==0 || length>entry.size_bytes{
                return Err(LpmError::Spft(format!(
                    "{} 이미지 크기 {}B는 scatter 허용 크기 {}B를 초과하거나 비어 있습니다.",
                    entry.name,length,entry.size_bytes
                )));
            }
        }
        result.push(entry.clone());
    }
    Ok(result)
}
fn readback_filename(name:&str,suffix:usize)->String{
    if suffix==0{name.to_string()}else{format!("{name}_{suffix:03}")}
}
fn unique_folder(base:&Path)->Result<PathBuf> {
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    for idx in 0..1000{
        let path=base.join(format!("partition_batch_{stamp}_{idx}"));
        if !path.exists(){ fs::create_dir_all(&path)?;return Ok(path);}
    }
    Err(LpmError::Spft("임시 작업 경로 확보 실패".into()))
}
fn build_batch_config(flash_xml:&Path,log_dir:&Path,files:&[(String,PathBuf)])->Result<String>{
    let mut cfg=build_readback_config_xml(flash_xml,log_dir,Path::new("placeholder"));
    let from=cfg.find("    <READ-PARTITION ")
        .ok_or_else(||LpmError::Spft("원본 proinfo READ-PARTITION 템플릿 없음".into()))?;
    let to=from+cfg[from..].find("    <REBOOT ")
        .ok_or_else(||LpmError::Spft("원본 proinfo COLD-RESET 템플릿 없음".into()))?;
    let commands=files.iter().map(|(partition,file)|format!(
        "    <READ-PARTITION version=\"1.0\" partition=\"{}\">\n      <target-file version=\"1.0\" file_type=\"LOCAL_FILE\" file_name=\"{}\"/>\n    </READ-PARTITION>\n",
        xml_escape(partition),xml_escape(&file.display().to_string())
    )).collect::<String>();
    cfg.replace_range(from..to,&commands);
    Ok(cfg)
}
fn build_batch_scatter(source:&str, images:&[(String,PathBuf)])->Result<String>{
    let mut result=String::with_capacity(source.len());
    let mut rest=source;
    let mut matches=HashSet::new();
    while let Some(start)=rest.find("<partition_index "){
        result.push_str(&rest[..start]);
        rest=&rest[start..];
        let end=rest.find("</partition_index>").ok_or_else(||
            LpmError::Spft("scatter XML 파티션 종료 오류".into()))?
            +"</partition_index>".len();
        let mut block=rest[..end].to_string();
        let name=xml_tag(&block,"partition_name").unwrap_or_default().to_string();
        if let Some((_,file))=images.iter().find(|(selected,_)|selected==&name){
            matches.insert(name);
            let file_name=file.file_name().ok_or_else(||
                LpmError::Spft("작업 이미지 경로 오류".into()))?.to_string_lossy();
            block=set_xml_tag(&block,"file_name",&file_name)?;
            block=set_xml_tag(&block,"is_download","true")?;
            block=set_xml_tag(&block,"operation_type","UPDATE")?;
        }else{
            block=set_xml_tag(&block,"is_download","false")?;
        }
        result.push_str(&block);
        rest=&rest[end..];
    }
    result.push_str(rest);
    if matches.len()!=images.len(){
        return Err(LpmError::Spft("작업용 scatter에 선택한 파티션 일부가 누락됨".into()));
    }
    Ok(result)
}
struct BatchPlan{
    command:Command,
    work_dir:PathBuf,
    outputs:Vec<(String,PathBuf,PathBuf,u64)>,
    writing:bool,
    timeout:Duration,
}
fn prepare_batch(image_path:&Path,selected:&[PartitionSelection],output_dir:Option<&Path>)
->Result<BatchPlan> {
    let writing=output_dir.is_none();
    let table=validate_selection(image_path,selected,writing)?;
    let (_,image_dir)=resolve_root_and_image_dir(image_path)?;
    let flash_xml=find_flash_xml(&image_dir).ok_or_else(||
        LpmError::FileNotFound("flash.xml".into()))?;
    require_spft_ascii_path(&flash_xml,"SPFT flash.xml")?;
    let spft_dir=ensure_embedded_spflashtoolv6_tool()?;
    let exe=spft_dir.join("SPFlashToolV6.exe");
    if !exe.is_file(){return Err(LpmError::FileNotFound(exe.display().to_string()));}
    let work_dir=unique_folder(&ascii_readback_workspace()?)?;
    let mut command=Command::new(exe);
    command.current_dir(&spft_dir);
    let mut outputs=Vec::new();
    // The legacy proinfo readback completes quickly, but a full userdata
    // selection can take much longer. Bound the batch timeout according
    // to the user's selected data size; no fixed 10-minute limit.
    let total_bytes=table.iter().fold(0u64,|sum,item|
        sum.saturating_add(item.size_bytes));
    let seconds=(total_bytes/(8*1024*1024)).saturating_add(600).clamp(600,14400);
    let timeout=Duration::from_secs(seconds);
    if let Some(destination)=output_dir{
        fs::create_dir_all(destination)?;
        let spool=work_dir.join("output");
        fs::create_dir_all(&spool)?;
        let mut names=Vec::new();
        for item in &table {
            let filename=readback_filename(&item.name,0);
            let mut final_path=destination.join(&filename);
            for suffix in 1..1000{
                if !final_path.exists(){break;}
                final_path=destination.join(readback_filename(&item.name,suffix));
            }
            if final_path.exists(){return Err(LpmError::Spft("덤프 파일명 충돌".into()));}
            let spool_path=spool.join(&filename);
            names.push((item.name.clone(),spool_path.clone()));
            outputs.push((item.name.clone(),spool_path,final_path,item.size_bytes));
        }
        let log_dir=work_dir.join("logs");
        fs::create_dir_all(&log_dir)?;
        let config_path=work_dir.join("batch_readback_config.xml");
        fs::write(&config_path,build_batch_config(&flash_xml,&log_dir,&names)?)?;
        command.arg("-i").arg(&config_path);
    }else{
        let scatter=find_scatter_xml(&image_dir)?;
        let source_flash=fs::read_to_string(&flash_xml)?;
        let dagent=xml_tag(&source_flash,"dagent").ok_or_else(||
            LpmError::Spft("flash.xml DA가 없습니다.".into()))?;
        if dagent.contains('/')||dagent.contains('\\')||dagent.contains(".."){
            return Err(LpmError::Spft("잘못된 DA 파일 경로".into()));
        }
        let orig_da=flash_xml.parent().ok_or_else(||
            LpmError::Spft("원본 DA 경로 오류".into()))?.join(dagent);
        if !orig_da.is_file(){return Err(LpmError::FileNotFound(orig_da.display().to_string()));}
        let agent_dir=work_dir.join("download_agent");
        fs::create_dir_all(&agent_dir)?;
        let mut staged=Vec::new();
        for item in selected {
            let path=work_dir.join(format!("partition_{}.img",item.name));
            fs::copy(item.image.as_ref().unwrap(),&path)?;
            staged.push((item.name.clone(),path));
        }
        let flash_body=fs::read_to_string(&flash_xml)?;
        let expected=format!("../{}",scatter.file_name()
            .unwrap_or_default().to_string_lossy());
        if xml_tag(&flash_body,"scatter")!=Some(expected.as_str()){
            return Err(LpmError::Spft("flash.xml의 scatter 경로 예상 형식 불일치".into()));
        }
        let rewritten=build_batch_scatter(&fs::read_to_string(&scatter)?,&staged)?;
        fs::write(work_dir.join(scatter.file_name().unwrap_or_default()),rewritten)?;
        fs::copy(&flash_xml,agent_dir.join("flash.xml"))?;
        fs::copy(&orig_da,agent_dir.join(dagent))?;
        command.arg("-a").arg(agent_dir.join(dagent))
            .arg("-f").arg(agent_dir.join("flash.xml")).arg("-c").arg("download");
    }
    Ok(BatchPlan{command,work_dir,outputs,writing,timeout})
}

pub struct PrearmedPartitionBatch{
    worker:thread::JoinHandle<Result<Vec<(String,PathBuf,u64)>>>,
    events:mpsc::Receiver<ProinfoLiveEvent>,
    cancel:std::sync::Arc<AtomicBool>,
}
impl PrearmedPartitionBatch {
    pub fn start(image_dir:&Path,selected:&[PartitionSelection],output_dir:Option<&Path>)->Result<Self>{
        let plan=prepare_batch(image_dir,selected,output_dir)?;
        let (events_tx,events)=mpsc::channel();
        let (ready_tx,ready_rx)=mpsc::sync_channel(1);
        let cancel=std::sync::Arc::new(AtomicBool::new(false));
        let thread_cancel=std::sync::Arc::clone(&cancel);
        let worker=thread::Builder::new().name("lpmbox-partition-batch".into()).spawn(move||{
            let BatchPlan{command,work_dir,outputs,writing,timeout}=plan;
            let result=run_spflashtoolv6_streaming(command,&mut |event|{
                let _=events_tx.send(event);
            },Some(thread_cancel.as_ref()),Some(ready_tx),Some(timeout))?;
            if !result.success{
                return Err(LpmError::Spft(format!(
                    "SPFlashToolV6 파티션 작업 실패: exit={:?}; stderr={}",
                    result.exit_code,result.stderr_tail
                )));
            }
            let mut finished=Vec::new();
            if writing{
                finished.push(("파티션 플래싱 완료".into(),work_dir,0));
            }else{
                for (name,spool,final_path,expected) in outputs{
                    let length=fs::metadata(&spool).map_err(|_|LpmError::Spft(format!(
                        "{name} 덤프 임시 파일이 존재하지 않습니다."
                    )))?.len();
                    if length!=expected{
                        return Err(LpmError::Spft(format!(
                            "{name} 덤프 길이 검증 실패: 실제 {length}B / Scatter 크기 {expected}B. 미완성 백업 파일은 저장하지 않았습니다."
                        )));
                    }
                    // Never overwrite an existing backup, including files created
                    // by another process after the name was initially selected.
                    let mut src=fs::File::open(&spool)?;
                    let mut dst=fs::OpenOptions::new().write(true).create_new(true)
                        .open(&final_path).map_err(|err|LpmError::Spft(format!(
                            "{name} 백업 경로가 이미 존재하거나 쓰기 실패: {} / {err}",
                            final_path.display()
                        )))?;
                    let copy=std::io::copy(&mut src,&mut dst);
                    match copy {
                        Ok(n) if n==expected=>{},
                        Ok(n)=>{
                            drop(dst);
                            let _=fs::remove_file(&final_path);
                            return Err(LpmError::Spft(format!(
                                "{name} 복사 길이 불일치: {n}B / {expected}B"
                            )));
                        }
                        Err(err)=>{
                            drop(dst);
                            let _=fs::remove_file(&final_path);
                            return Err(LpmError::Spft(format!(
                                "{name} 백업 저장 중 오류: {err}"
                            )));
                        }
                    }
                    finished.push((name,final_path,length));
                }
            }
            Ok(finished)
        }).map_err(|e|LpmError::Spft(format!("SPFT 작업 스레드 시작 실패: {e}")))?;
        match ready_rx.recv_timeout(Duration::from_secs(12)){
            Ok(Ok(()))=>{
                thread::sleep(Duration::from_millis(250));
                if worker.is_finished(){
                    let out=worker.join().map_err(|_|LpmError::Spft("SPFT 작업 스레드 종료".into()))?;
                    return Err(LpmError::Spft(format!(
                        "SPFT가 USB 대기 단계 전에 종료되었습니다: {:?}",out.err()
                    )));
                }
            }
            Ok(Err(e))=>{let _=worker.join();return Err(LpmError::Spft(e));}
            Err(_)=>{
                cancel.store(true,Ordering::Release);
                let _=worker.join();
                return Err(LpmError::Spft("SPFT USB 대기 준비 시간 초과".into()));
            }
        }
        Ok(Self{worker,events,cancel})
    }
    pub fn finish<F>(self,mut on_event:F)->Result<Vec<(String,PathBuf,u64)>>
    where F:FnMut(ProinfoLiveEvent){
        while !self.worker.is_finished(){
            match self.events.recv_timeout(Duration::from_millis(75)){
                Ok(event)=>on_event(event),
                Err(mpsc::RecvTimeoutError::Timeout)=>{},
                Err(mpsc::RecvTimeoutError::Disconnected)=>break,
            }
        }
        while let Ok(event)=self.events.try_recv(){on_event(event);}
        self.worker.join().map_err(|_|LpmError::Spft("SPFT 작업 스레드 패닉".into()))?
    }
    pub fn cancel(self){
        self.cancel.store(true,Ordering::Release);
        let _=self.finish(|_|{});
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    const SAMPLE:&str=r#"<root><partition_index name="SYS0"><partition_name>boot_a</partition_name><file_name>boot.img</file_name><is_download>true</is_download><operation_type>UPDATE</operation_type><physical_start_addr>0x2000</physical_start_addr><partition_size>0x1000</partition_size><region>EMMC_USER</region></partition_index><partition_index name="SYS1"><partition_name>boot_b</partition_name><file_name>boot_b.img</file_name><is_download>true</is_download><operation_type>UPDATE</operation_type><partition_size>0x1000</partition_size><region>EMMC_USER</region></partition_index></root>"#;
    #[test]
    fn readback_files_are_extensionless(){
        assert_eq!(readback_filename("boot_a",0),"boot_a");
        assert_eq!(readback_filename("userdata",1),"userdata_001");
        assert_eq!(readback_filename("proinfo",25),"proinfo_025");
    }
    #[test]
    fn enumerate_scatter_partitions(){
        let entries=scan_scatter(SAMPLE).unwrap();
        assert_eq!(entries.len(),2);
        assert_eq!(entries[0].name,"boot_a");
        assert_eq!(entries[0].size_bytes,4096);
    }
    #[test]
    fn multi_flash_only_selected_entries(){
        let targets=vec![("boot_a".into(),PathBuf::from("G:/tmp/boot_a.img"))];
        let out=build_batch_scatter(SAMPLE,&targets).unwrap();
        assert_eq!(out.matches("<is_download>true</is_download>").count(),1);
        assert_eq!(out.matches("<is_download>false</is_download>").count(),1);
    }
    #[test]
    fn disallows_sensitive_and_b_slot_writes(){
        assert!(!approved_flash_partition("proinfo"));
        assert!(!approved_flash_partition("lk_b"));
        assert!(!approved_flash_partition("preloader"));
        assert!(approved_flash_partition("boot_a"));
    }

    #[test]
    fn readback_batch_has_only_the_requested_partition_labels() {
        let files=vec![
            ("boot_a".to_string(), PathBuf::from(r"C:\Temp\boot_a.img")),
            ("lk_a".to_string(), PathBuf::from(r"C:\Temp\lk_a.img")),
        ];
        let xml=build_batch_config(Path::new(r"C:\ROM\image\download_agent\flash.xml"),
            Path::new(r"C:\Temp\logs"),&files).unwrap();
        assert_eq!(xml.matches("<READ-PARTITION version=").count(),2);
        assert!(xml.contains("partition=\"boot_a\""));
        assert!(xml.contains("partition=\"lk_a\""));
        assert!(!xml.contains("partition=\"proinfo\""));
        assert_eq!(xml.matches("<REBOOT version=").count(),1);
        assert_eq!(xml.matches("<target-file version=").count(),2);
    }
}

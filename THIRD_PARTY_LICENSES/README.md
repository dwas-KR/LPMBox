# License and asset attribution for LPMBOX

The project's Cargo manifests declare GPL-3.0-only for the author's
licensable original Rust code. The root LICENSE is GNU GPL version 3.
This statement does not relicense code, art, firmware or other components
owned by third parties.

## Previous LPMBOX distribution
- PRIOR_RELEASE_CC_BY_NC_SA_4.0.txt retains the CC BY-NC-SA 4.0
  license published with earlier releases. This is historical context,
  **not** a second current root project license.
- If any source from earlier releases or LTBox was adapted under that
  noncommercial ShareAlike license, relicensing it under GPLv3
  requires the relevant copyright holders' permission. A permission
  merely to develop does not necessarily grant this right.
- The included notice does not establish ownership or relicensing rights.

## Flags, fonts, and icons
- Country SVG flags: flag-icons, copyright (c) 2013 Panayiotis Lipiridis,
  MIT license; preserve crates/lpmbox-gui/assets/flags/LICENSE beside the SVGs.
- crates/lpmbox-gui/assets/fonts/Inter.ttf: Inter Project Authors,
  SIL Open Font License 1.1. See Inter-OFL-1.1-LICENSE.txt. The font's
  internal name and license metadata identify it as Inter, and the copyright
  notice is retained.
- Navigation SVG icon shapes based on Lucide/Feather: see
  Lucide-ISC-LICENSE.txt and the upstream Lucide project. Derived icons
  retain any upstream attribution requirements.
- Some raster icons list GPT generation as their source; authorship and
  third-party resemblance still require a separate rights check.

## LK and vendor images (redistribution review required)
- third_party/TB375FC-LK-patch-LICENSE.txt is the MIT license copied
  with the ifg patch-tool research. The license itself expressly limits
  its scope to tool source code, **not** Lenovo firmware or images
  produced from it.
- The included *.img LK binaries contain bootloader/firmware-derived
  bytes. A SHA-256 match or successful device boot does not grant permission
  for redistribution. Obtain the relevant rights or another authorized
  distribution approach before public repository upload.
- crates/lpmbox-gui/assets/models/*.png lists product images sourced
  from Lenovo's website. The product pages do not by themselves establish
  a redistribution license for rehosting those images.
- The external SP Flash Tool binary is not included in this source tree.
  See crates/lpmbox-spft/assets/SPFlashToolV6_tool.txt.

## Source attribution
- The project's README credits the LTBox authors and links to the
  independent upstream project. Ensure any copied implementation,
  artwork and licenses have documented provenance.
- The licenses accompanying unrelated crates remain governed by their
  own package metadata; the source workspace does not relicense them.

**Public distribution is not legally verified** until the old
CC BY-NC-SA material (if still present), vendor-derived LK files, and
Lenovo product images are cleared or lawfully replaced.

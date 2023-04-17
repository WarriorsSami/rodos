use crate::application::Void;
use crate::core::config::Config;
use crate::core::Arm;
use crate::domain::i_disk_manager::IDiskManager;
use color_print::cprintln;
use mediator::{Request, RequestHandler};

// NeofetchEvent is a request for retrieving specifications about the OS.
pub(crate) struct NeofetchRequest;

impl NeofetchRequest {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Request<Void> for NeofetchRequest {}

// NeofetchHandler is a handler for NeofetchRequest.
pub(crate) struct NeofetchHandler {
    config: Arm<Config>,
    disk_manager: Arm<dyn IDiskManager>,
}

impl NeofetchHandler {
    pub(crate) fn new(config: Arm<Config>, disk_manager: Arm<dyn IDiskManager>) -> NeofetchHandler {
        NeofetchHandler {
            config,
            disk_manager,
        }
    }
}

impl RequestHandler<NeofetchRequest, Void> for NeofetchHandler {
    fn handle(&mut self, _event: NeofetchRequest) -> Void {
        log::info!("Showing OS specifications...");

        match self.config.lock() {
            Ok(config) => {
                let boot_sector = self.disk_manager.lock().unwrap().get_boot_sector();

                cprintln!("<bold>
                    <w!>WWWWWWWWWWWX</><c!>Okk0</><w!>XWWXXK</><c!>00000</><w!>KNWWWWWWWWWWW</>            <w!>{}</><b!>{}</><w!>{}</>
                    <w!>WWWWWWWN</><c!>0xlcok</><w!>XWWK</><c!>xooolllllodxO</><w!>KNWWWWWWW</>            <k!>----------------</>
                    <w!>WWWWWNkl</><b!>;,</><w!>o0NWWW</><c!>0ololllolllllllox0</><w!>NWWWWW</>            <r!>OS</>: <w!>{} {}</>
                    <w!>WWWNOc</><b!>,':</><w!>kNWWWWN</><c!>xllllllokxlllloolld</><w!>KWWWW</>            <r>Author</>: <w!>{}</>
                    <w!>WWXd;</><b!>,,</><w!>c0WWWWWWW</><c!>Oolllllx</><w!>XN00000000</><c!>Ok0</><w!>NWW</>            <r>FAT</>: <w!>FAT{}</>
                    <w!>WXo</><b!>,,,:</><w!>0WWWWWWWWN</><c!>KOkkk</><w!>0NWKkdlcccloxOKNWW</>            <r>No Clusters</>: <w!>{}</>
                    <w!>Nd</><b!>,,,,</><w!>xWWWWWWWWWWWWWWWWNx:,'</><b!>,,,,,,',</><w!>ckNW</>            <r>Cluster Size</>: <w!>{} bytes</>
                    <w!>0:</><b!>',':</><w!>0</><c!>N</><w!>NWWWWWWWWWWWWWWk;'</><b!>,,,,</><w!>::</><b!>,,,,,,</><w!>oX</>            <r>Disk Size</>: <w!>{} bytes</>
                    <w!>x</><b!>,,,'</>:<w!>0</><c!>KkX</><w!>WWWWWWWWWWWWWk</><b!>,,,,,,</><w!>dk:</><b!>',,,'</><w!>,x</>
                    <w!>o</><b!>,,,'</><w!>;O</><c!>XddK</><w!>NWWWWWWWWW</><c!>WM</><w!>Xd</><b!>:,,;</><w!>lKKc'</><b!>,,,,</><w!>'o</>
                    <w!>d</><b!>,,,,,</><w!>l</><c!>K0oox0</><w!>KXNWNNN</><c!>XX0O</><w!>00OO</><c!>OK</><w!>WO;'</><b!>,,,,,</><w!>o</>
                    <w!>k;</><b!>',,,,</><w!>oK</><c!>0dlloddxxddoollodddk</><w!>K0c</><b!>,,,,,</><w!>',k</>
                    <w!>Kl</><b!>',,,,,</><w!>lO</><c!>Kkdolllllllllllod</><w!>OKk:</><b!>,,,,,,</><w!>'cK</>
                    <w!>WO:</><b!>,,,,,,</><w!>;lk</><c!>00OkxdddddxkO0</><w!>0kl</><b!>,,,,,,,</><w!>':OW</>
                    <w!>WWO:</><b!>,,,,,,',</><w!>:oxkOOOOOOkkdl</><b!>;,,,',,,,,</><w!>:OWW</>
                    <w!>WWWKl</><b!>,',,,,,,,',,,,,,,,'',,,,,,,,,,</><w!>l0WWW</>
                    <w!>WWWWNkc</><b!>,,,,,,,,,,,,,,,,,,,,,,,,,,</><w!>ckNWWWW</>
                    <w!>WWWWWWNOo</><b!>:,'',,,,,,,,,,,,,,,',</><w!>:oONWWWWWW</>
                    <w!>WWWWWWWWWXOxoc;</><b!>,,,,,,,,,,;</><w!>coxOXWWWWWWWWW</>
                    <w!>WWWWWWWWWWWWWX0kxdoooodxk0XWWWWWWWWWWWWW</>
                    </>",
                    config.prompt.host,
                    config.prompt.separator,
                    config.prompt.user,
                    config.os,
                    config.version,
                    config.author,
                    boot_sector.cluster_size,
                    boot_sector.cluster_count,
                    boot_sector.cluster_size,
                    boot_sector.cluster_size as u32 * boot_sector.cluster_count as u32,
                );
                log::info!("Showing OS specifications... done");

                Ok(())
            }
            Err(_) => Err(Box::try_from("Unable to lock config").unwrap()),
        }
    }
}

use crate::application::create::CreateRequest;
use crate::application::Void;

pub(crate) trait IDiskManager: Sync + Send {
    fn push_sync(&mut self);
    fn pull_sync(&mut self);
    fn create_file(&mut self, request: CreateRequest) -> Void;
}

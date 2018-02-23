use std::result;

type Result = result::Result<(),()>;
trait PermissionHolder {
    fn test_perm(&self, perm: Permission) -> Result;
}

#[derive(Debug)]
struct Permission {

}
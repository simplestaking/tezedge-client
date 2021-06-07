use types::NewOperationGroup;
use crate::BoxFuture;

pub type ForgeOperationsResult = Result<String, ()>;

pub trait ForgeOperations {
    fn forge_operations(
        &self,
        operation_group: &NewOperationGroup,
    ) -> ForgeOperationsResult;
}

pub trait ForgeOperationsAsync {
    fn forge_operations<'a>(
        &'a self,
        operation_group: &'a NewOperationGroup,
    ) -> BoxFuture<'a, ForgeOperationsResult>;
}

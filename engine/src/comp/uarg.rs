use std::mem::ManuallyDrop;

pub enum UArgType
{
    INT,
    UINT,
    INT64,
    UINT64,
    FLOAT,
    DOUBLE,
    STRING,
}
pub union UArgVal {
    int: i32,
    uint: u32,
    int64: i64,
    uint64: u64,
    float: f32,
    double: f64,
    string: ManuallyDrop<String>,
}
pub struct UArg
{
    name: String,
    kind: UArgType,
    val: UArgVal,
}

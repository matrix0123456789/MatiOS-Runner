use crate::uuid::Uuid;

pub struct HostMachine{

}
impl HostMachine{
    pub fn getCurrentHostMachineGuid() -> Uuid {
     return Uuid::from_u128(0x00000000_0000_0000_0000_100000000000);//tmp static guid
    }
}
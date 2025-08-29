pub trait GameType {
    fn get_name(&self) -> String;
    fn get_platform(&self) -> String;
    fn get_platform_id(&self) -> String;
    fn get_description(&self) -> String;
    fn get_price(&self) -> String;
}
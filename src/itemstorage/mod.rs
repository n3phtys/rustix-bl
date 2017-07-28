

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct User {
    username: String,
    external_user_id: u32,
    user_id: u32,
    subuser_to: Option<u32>,
    is_billed: bool,
}



#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Item {
    name: String,
    item_id: u32,
    category_id: u32,
    cost_euros: u8,
    cost_cents: u8,
}

pub fn test() {

    let x = UserBuilder::default()
        .external_user_id(19124u32)
        .build()
        ;
    println!("{:?}", x);

    let y = ItemBuilder::default()
        .name("cool item")
        .cost_euros(42u8)
        .item_id(19124u32)
        .build()
        ;
    println!("{:?}", y);
}
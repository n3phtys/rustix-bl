#[macro_use] extern crate derive_builder;

mod itemstorage;

#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
struct Channel {
    token: i32,
    special_info: i32,
    // .. a whole bunch of other fields ..
}


fn main() {
    println!("Hello, world!");

    itemstorage::test();

    // builder pattern, go, go, go!...
    let ch = ChannelBuilder::default()
        .special_info(42u8)
        .token(19124)
        .build()
        .unwrap();
    println!("{:?}", ch);

    rustix::test();
}

#[macro_use]
mod rustix {

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
        .user_id(19124u32)
        .build()
        ;
        println!("{:?}", x);

        let y = ItemBuilder::default()
        .name("cool item")
        .cost_euros(42u8)
        .item_id(19124u32)
        .build();
    println!("{:?}", y);
    }
}
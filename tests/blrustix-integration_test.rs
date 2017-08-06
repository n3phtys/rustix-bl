extern crate blrustix;
extern crate tempdir;

use blrustix::rustix_backend::WriteBackend;


#[test]
fn it_add_user() {
    let dir = tempdir::TempDir::new("temptestdir").unwrap();

    let b = blrustix::build_persistent_backend(dir.as_ref());

    println!("{:?}", b);

    match b {
        Err(_) => assert!(false),
        Ok(mut backend) => {
            backend.create_user("klaus".to_string());
            assert_eq!(
                backend.datastore.users.get(&0u32).unwrap().username,
                "klaus".to_string()
            );
        }
    }
}


#[test]
fn it_reload_added_user() {


    /*{


        let dir = std::path::Path::new("target");

        let b = blrustix::build_persistent_backend(dir.as_ref());

        println!("{:?}", b);

        match b {
            Err(_) => assert!(false),
            Ok(mut backend) => {
                backend.create_user("klaus".to_string());
                assert_eq!(
                    backend.datastore.users.get(&0u32).unwrap().username,
                    "klaus".to_string()
                );
            }
        }

    }*/




    let dir = std::path::Path::new("tests/testdata");

    {
        match blrustix::build_persistent_backend(dir) {
            Err(_) => assert!(false),
            Ok(mut backend) => {
                let x = backend.reload();
                println!("Loaded Backend: {:?}", backend);
                assert_eq!(x.unwrap(), 1);
                assert_eq!(
                    backend.datastore.users.get(&0u32).unwrap().username,
                    "klaus".to_string()
                );
            }
        }
    }
}

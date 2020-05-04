extern crate file_system;
use file_system::disk::Disk;
use std::fs;
use std::io::*;

fn main() {
    let mut file_name = String::new();
    let mut disk_mount: Option<Disk> = None;

    loop {
        println!("Enter 1 to create a disk");
        println!("Enter 2 to mount disk");
        println!("Enter 3 to unmount disk");
        println!("Enter 4 to ls directory");
        println!("Enter 5 to get diagnostic information");
        println!("Enter 6 to quit");
        println!("Enter 7 for help");
        println!("Enter 8 to write file to disk");
        println!("Enter 9 to format disk");
        let mut input = String::new();
        let mut input_two = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut input)
            .expect("Could not read user input");
        input = remove_carriage_return(input);
        let my_int = input.parse().expect("Should of unwrawpped");

        match my_int {
            1 => {
                input = String::new();
                input_two = String::new();
                println!("Please enter a path for the disk");
                stdin()
                    .read_line(&mut input)
                    .expect("Could not read user input");
                println!("Please enter a disk size");
                stdin()
                    .read_line(&mut input_two)
                    .expect("Could not read user input");
                input = remove_carriage_return(input);
                input_two = remove_carriage_return(input_two);
                let my_u32 = input_two.parse().unwrap();
                let disk_is_created = file_system::FileSystem::create_disk(input, my_u32);
                println!("{}", disk_is_created);
            }

            2 => {
                file_name = String::new();
                println!("name of disk to be mounted");
                stdin()
                    .read_line(&mut file_name)
                    .expect("Could not read user input");
                file_name = remove_carriage_return(file_name);
                println!("{}", file_name);
                disk_mount = file_system::FileSystem::mount(&file_name);
                println!("{:?}", disk_mount);
            }

            3 => {
                disk_mount = None;
                println!("Unmounted disk");
            }

            4 => {
                disk_mount
                    .and_then(|disk| {
                        let (d, disk) = file_system::FileSystem::get_directory()(disk);
                        disk_mount = Some(disk);
                        d
                    })
                    .map(|d| {
                        println!("{:?}", d);
                        d
                    });
            }

            5 => {
                println!("Disk diagnostics");
                disk_mount
                    .and_then(|disk| {
                        let (d, disk) = file_system::FileSystem::get_diagnostic()(disk);
                        disk_mount = Some(disk);
                        d
                    })
                    .map(|d| {
                        println!("{:?}", d);
                        d
                    });
            }

            6 => {
                let exit_code = 0;
                println!("Exiting");
                std::process::exit(exit_code);
            }

            7 => {
                help();
            }

            8 => {
                input = String::new();
                input_two = String::new();
                println!("Please enter the disk to be written to");
                stdin()
                    .read_line(&mut input)
                    .expect("Could not read user input");
                input = remove_carriage_return(input);
                println!("Please enter file name");
                stdin()
                    .read_line(&mut input_two)
                    .expect("Could not read user input");
                input_two = remove_carriage_return(input_two);
                let file_data = fs::read_to_string(&input_two).unwrap();
                fs::write(input, &file_data).unwrap();
                disk_mount
                    .map(|disk| {
                        let (data, disk) =
                            file_system::FileSystem::save_as_file(input_two, file_data)(disk);
                        disk_mount = Some(disk);
                        data
                    })
                    .map(|x| match x {
                        Some(_) => println!("Save file successfully"),
                        None => println!("Something went wrong"),
                    });
            }

            9 => {
                input = String::new();
                input_two = String::new();
                println!("Please enter disk to format");
                stdin()
                    .read_line(&mut input)
                    .expect("Could not read user input");
                input = remove_carriage_return(input);
                println!("Please enter new disk size");
                stdin()
                    .read_line(&mut input_two)
                    .expect("Could not read user input");
                input_two = remove_carriage_return(input_two);
                let parsed = input_two.parse().unwrap();
                let formatted = file_system::FileSystem::format(input, parsed);
                println!("{:?}", formatted);
            }
            _ => println!("Something went wrong"),
        }
    }
}

pub fn help() {
    println!("To create a disk path enter a path like ./sda1 or ./sda2");
    println!("Enter a disk size such as 30 or 10");
    println!("To mount a disk, enter the disk path as you did earlier, for example ./sda1");
}

pub fn remove_carriage_return(mut input: String) -> String {
    input.pop();
    // input.pop();
    input
}

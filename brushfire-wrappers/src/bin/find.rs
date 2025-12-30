use brushfire_wrappers::{
    executor::exec_real_utility, policy_loader::get_policy_engine, arg_parser::parse_args,
    print_policy_error, schemas::FIND_SCHEMA,
};
use std::path::Path;

fn main() {
    let policy = match get_policy_engine() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("brushfire: Failed to load policy: {}", e);
            std::process::exit(2);
        }
    };

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut file_args = parse_args(&FIND_SCHEMA, &args);

    // If no file arguments, find defaults to current directory
    if file_args.is_empty() {
        file_args.push(brushfire_wrappers::schemas::FileArg::new(
            ".".to_string(),
            brushfire_policy::FileAccessMode::Read,
        ));
    }

    for file_arg in file_args {
        let path = Path::new(&file_arg.path);
        if let Err(e) = policy.check_file_access(path, file_arg.access_mode) {
            print_policy_error(FIND_SCHEMA.name, path, &e);
            std::process::exit(1);
        }
    }

    exec_real_utility(&FIND_SCHEMA, &args);
}

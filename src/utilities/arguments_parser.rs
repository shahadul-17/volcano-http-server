use std::collections::HashMap;
use std::env;

const ARGUMENT_NAME_PREFIX: &str = "--";
const ARGUMENT_NAME_PREFIX_LENGTH: usize = ARGUMENT_NAME_PREFIX.len();

pub fn populate_arguments_map() -> HashMap<String, String> {
    let mut i = 1;
    let arguments: Vec<String> = env::args().collect();
    let mut arguments_map: HashMap<String, String> = HashMap::with_capacity(arguments.len());

    while i < arguments.len() {
        let argument = arguments.get(i).unwrap();

        i = i + 1;

        if !argument.starts_with(ARGUMENT_NAME_PREFIX) { continue; }

        let argument_name = (&argument[ARGUMENT_NAME_PREFIX_LENGTH..]).to_string();
        let argument_value = (arguments.get(i).unwrap()).to_string();              // next argument is the value...
        arguments_map.insert(argument_name, argument_value);
    }

    return arguments_map;
}

pub fn get_argument(argument_name: &str, arguments_map: &HashMap<String, String>) -> String {
    let argument_value_option = arguments_map.get(argument_name);

    if argument_value_option.is_none() { return String::from(""); }

    return argument_value_option.unwrap()[..].to_string();
}

pub fn get_argument_or_default(argument_name: &str, default_value: &str, arguments_map: &HashMap<String, String>) -> String {
    let argument_value = get_argument(argument_name, arguments_map);

    if argument_value.is_empty() {
        return String::from(default_value);
    }

    return argument_value;
}

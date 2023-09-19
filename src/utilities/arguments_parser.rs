use std::collections::HashMap;
use std::env;

const ARGUMENT_NAME_PREFIX: &str = "--";
const ARGUMENT_NAME_PREFIX_LENGTH: usize = ARGUMENT_NAME_PREFIX.len();

#[derive(Clone)]
pub struct ArgumentsParser {
    arguments_map: HashMap<String, String>,
}

impl ArgumentsParser {
    pub fn new(name_only_arguments: &Vec<String>) -> Self {
        let arguments_map = populate_arguments_map(name_only_arguments);
        let arguments_parser = ArgumentsParser {
            arguments_map,
        };

        return arguments_parser;
    }

    pub fn get_as_boolean(&self, argument_name: &str, default_value: &str) -> bool {
        let argument_value = self.get_as_string(argument_name, default_value);

        return "true" == argument_value;
    }

    pub fn get_as_u16(&self, argument_name: &str, default_value: &str) -> u16 {
        let argument_value = self.get_as_string(argument_name, default_value);
        let parse_result = argument_value.parse::<u16>();

        if parse_result.is_err() {
            let error = parse_result.unwrap_err();

            eprintln!("An error occurred while parsing {} ('{}') as u16: {}", argument_name, argument_value, error);

            return 0;
        }

        return parse_result.unwrap();
    }

    pub fn get_as_usize(&self, argument_name: &str, default_value: &str) -> usize {
        let argument_value = self.get_as_string(argument_name, default_value);
        let parse_result = argument_value.parse::<usize>();

        if parse_result.is_err() {
            let error = parse_result.unwrap_err();

            eprintln!("An error occurred while parsing {} ('{}') as usize: {}", argument_name, argument_value, error);

            return 0;
        }

        return parse_result.unwrap();
    }

    pub fn get_as_u64(&self, argument_name: &str, default_value: &str) -> u64 {
        let argument_value = self.get_as_string(argument_name, default_value);
        let parse_result = argument_value.parse::<u64>();

        if parse_result.is_err() {
            let error = parse_result.unwrap_err();

            eprintln!("An error occurred while parsing {} ('{}') as u64: {}", argument_name, argument_value, error);

            return 0;
        }

        return parse_result.unwrap();
    }

    pub fn get_as_string(&self, argument_name: &str, default_value: &str) -> String {
        let argument_value_option = self.arguments_map.get(argument_name);

        if argument_value_option.is_none() {
            return String::from(default_value);
        }

        let argument_value = argument_value_option.unwrap().to_owned();

        return argument_value;
    }
}

fn populate_arguments_map(name_only_arguments: &Vec<String>) -> HashMap<String, String> {
    let mut i = 1;
    let arguments: Vec<String> = env::args().collect();
    let mut arguments_map: HashMap<String, String> = HashMap::with_capacity(arguments.len());

    while i < arguments.len() {
        let argument = arguments.get(i).unwrap();

        i = i + 1;

        if !argument.starts_with(ARGUMENT_NAME_PREFIX) {
            continue;
        }

        let argument_name = (&argument[ARGUMENT_NAME_PREFIX_LENGTH..]).to_string();
        let argument_value: String;

        if name_only_arguments.contains(&argument_name) {
            argument_value = "true".to_string();
        } else {
            // otherwise, next argument is the value...
            let argument_value_option = arguments.get(i);

            // if argument value does not exist...
            if argument_value_option.is_none() {
                // we shall skip this iteration...
                continue;
            }

            argument_value = argument_value_option.unwrap().to_string();
        }

        arguments_map.insert(argument_name, argument_value);
    }

    return arguments_map;
}

const DATA_MEM_SIZE: i32 = 16;
const TEXT_MEM_SIZE: i32 = 16;

fn data_to_mem(data: &Vec<String>) -> String {
    let mut output_str: String = "v3.0 hex words addressed\n".to_string();
    let mut data_counter: i32 = 0;
    let mut mem_indexer: i32 = 0;
    let full_data_size = DATA_MEM_SIZE * DATA_MEM_SIZE;

    for (_, str) in data.iter().enumerate() {
        data_counter += 1;
        data_counter += str.chars().filter(|&c| c == ',').count() as i32;
    }

    for i in 0..full_data_size {
        if mem_indexer % 16 == 0 {
            output_str += format!("{:x}", mem_indexer).as_str();
        }

        let mut has_val: u8 = 0;

        if i <= data_counter {
            has_val += 1;
        }

        if has_val == 1 {
            // Get the translation into 1 byte in hex for that
        } else {
            output_str += "00"
        }

        if mem_indexer % 15 == 0 && mem_indexer != full_data_size {
            output_str += "\n";
        } else {
            output_str += " ";
        }

        mem_indexer += 1;
    }

    return output_str;
}

fn text_to_mem(text: &Vec<String>) -> String {
    let mut output_str: String = "v3.0 hex words addressed".to_string();
    let mut mem_indexer: i32 = 0;
    let full_data_size = TEXT_MEM_SIZE * TEXT_MEM_SIZE;

    for i in 0..full_data_size {
        if mem_indexer % 16 == 0 {
            output_str += format!("{:x}", mem_indexer).as_str();
        }

        let mut has_val: u8 = 0;

        if i <= text.len() as i32 {
            has_val += 1;
        }

        if has_val == 1 {
            // Get the translation into 1 byte in hex for that
        } else {
            output_str += "00000000"
        }

        if mem_indexer % 15 == 0 && mem_indexer != full_data_size {
            output_str += "\n";
        } else {
            output_str += " ";
        }

        mem_indexer += 1;
    }

    return output_str;
}

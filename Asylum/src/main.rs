//use std::collections::HashMap;
fn main() {
    println!("Hello, world!");

    recurse_and_rename("/a/b]/");


}


//one part of a path, not the whole path
fn strip_unwanted(input : &str) -> String{

    let mut buffer = String::with_capacity(input.len());


    let illegals = [
        //Windows Illegals (SMB)
        '[',
        ']',
        '!',
        '\\',
        ':',
        '<',
        '>',
        '*',
        '"',
        ';',
        '|',
        ',',
        '?',
        //Script baddies (bash/batch)
        '\'', //quote
        '@',
        '$',
        '+',
        '%',
        '-',
        '`',
        '#',
        '~',
        '^',
        '+',
        '='
    ];

    //repalce these 
    let replacer = '_';
    let replacers = [
        ' ',
        '(',
        ')'
    ];

    for c in input.chars(){
        if illegals.contains(&c)  {
            //do nothing
        } else if replacers.contains(&c){
            buffer.push(replacer);
        } else {
            //cannot end in a dot or a space
            buffer.push(c); 
        }
    }

    //prevent last character in filename 
    let cant_enders = [
        '.',
        ' '
    ];
    //Windows file rules say can't end in space or dot
    if let Some(x) = buffer.chars().last() {
        if cant_enders.contains(&x) {
            buffer.pop();
        }
    }
    buffer

}

fn recurse_and_rename(file_path : &str){

    let splitter = '/';

    let path_parts = file_path.split(splitter);

    for path in path_parts {
        println!("{}",strip_unwanted(path));
    }
    


}
use walkdir::WalkDir;
use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::path::PathBuf;

fn visit_dirs(dir: &Path, callback: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, callback)?;
                callback(&entry);
            } else {
                callback(&entry);
            }
        }
    }
    Ok(())
}

fn visit_dirs_sorted(dir: &Path, callback: &dyn Fn(&Path)) -> io::Result<()> {
    if dir.is_dir() {
        let dir_entries = fs::read_dir(dir)?;
        let mut entries : Vec<PathBuf> = dir_entries
            .filter(Result::is_ok)
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();

        for entry in entries {
            let path = entry.as_path();
            if path.is_dir() {
                visit_dirs_sorted(&path, callback)?;
                callback(&entry);
            } else {
                callback(&entry);
            }
        }
    }
    Ok(())
}


fn print_if_file(entry: &DirEntry) {
    let path = entry.path();
    if !path.is_dir() {
        println!("{}", path.to_string_lossy())
    }
}

fn print_if_dir(entry: &DirEntry) {
    let path = entry.path();
    if path.is_dir() {
        println!("{}", path.to_string_lossy())
    }

}

fn main() {
    println!("Starting Asylum");
    let path = std::env::args().nth(1);
    let path = match path {
        Some(x) => { x}
        //todo remove this was for debugging
        None => {String::from("/home/user/a")}
    };
    let path = std::path::Path::new(&path);
    //recurse_and_rename(&path);

    let result = visit_dirs(&path, &|file_or_dir| println!("{:?}", file_or_dir));

    let result2 = visit_dirs_sorted(&path, &|file_or_dir| println!("{:?}",file_or_dir));
}

//struct arg_options{

//    bool delete_empty/_folders;

//}

//one part of a path, not the whole path
fn strip_unwanted(input : &str) -> String{

    let mut buffer = String::with_capacity(input.len());



    let white_list = [
        'a','A',
        'b','B',
        'c','C',
        'd','D',
        'e','E',
        'f','F',
        'g','G',
        'h','H',
        'i','I',
        'j','J',
        'k','K',
        'l','L',
        'm','M',
        'n','N',
        'o','O',
        'p','P',
        'q','Q',
        'r','R',
        's','S',
        't','T',
        'u','U',
        'v','V',
        'w','W',
        'x','X',
        'y','Y',
        'z','Z',
        '0','1',
        '2','3',
        '4','5',
        '6','7',
        '8','9',
        ',','.',
        '_'
    ];

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
            //found illegal character
        } else if replacers.contains(&c){
            buffer.push(replacer);
        } else if white_list.contains(&c){
            buffer.push(c); 
        } else {
            //found non whitelisted char
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

fn recurse_and_rename(base_path : &std::path::Path){
    println!("With Base path:{}",base_path.to_string_lossy());
    if base_path.is_dir() {
        //for entry in std::fs::read_dir(base_path) {
        let entries = WalkDir::new(&base_path).contents_first(true);
        //entries.reverse();
        for entry in  entries {
            match entry {
                Ok(entry) => {
                    let dir_entry : walkdir::DirEntry = entry;
                    //let path : std::path::Path = entry.path();
                    let path = dir_entry.path();
                    println!("is Dir: {}" , path.is_dir());
                    println!("Full Path is: {}",path.display());
                    println!("last comp: {:?}", path.components().last().expect("expected content for last") );
                    let x = path.components().last().expect("expected content for last");
                    //let path_string : String = match x {
                    //    Normal(x) => x
                    //};
                    let filename = path.components().as_path().file_name().expect("can not self");
                    println!("filename: {:?}", filename);
                    let a = path.components().as_path().parent().expect("can not parent");
                    println!("parent: {:?}", a);

                },
                Err(_) => {
                    println!("{}","skip item. Probably a premissions issue");
                },
            }
            //let entry = entry.unwrap();
            

            //let entry = entry?;

        }
    }
    println!("{}","Read dir:");

//    visit_dirs(&base_path, cb);
    let dir = base_path;
    
    if dir.is_dir() {
        //for entry in fs::read_dir(dir).unwrap() {
        let mut paths: Vec<_> = fs::read_dir(dir).unwrap().map(|red| red.unwrap().path()).collect();
        paths.sort();
        for path in paths {
            //let path = entry.unwrap().path();
            if path.is_dir() {
                println!("is dir: {}",path.display());
            } else {
                println!("is file: {}",path.display());
            }
    
        }
    }
}

    //let mut path_buf = std::path::PathBuf::from(file_path);
    //for comp in path_buf.components() {
    //    match comp {
    //        println!("{:?}", x);
    //    }
    //}



    //let splitter = std::path::MAIN_SEPARATOR;
    
    //let path_parts = file_path.split(splitter);

    //for path in path_parts {
      //  println!("{}",strip_unwanted(path));
    //}
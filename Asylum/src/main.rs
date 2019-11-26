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
        //let mut dir_entries : Vec<PathBuf> = ;
        let mut entries : Vec<PathBuf> = fs::read_dir(dir)
            .expect("cannot read directory")
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
    let result = visit_dirs(
        &path, 
        &|file_or_dir| println!("{:?}", file_or_dir)
    );
    let result2 = visit_dirs_sorted(
        &path, 
        &|file_or_dir| println!("{:?}",file_or_dir)
    );
}

fn strip_unwated(path_buf : &PathBuf, behaviors: &Behaviors) {

    let mut input  = path_buf
        .file_name()
        .expect("path was expected")
        .to_string_lossy();
    
    match strip_unwanted(&input, &behaviors) {
        Changeable::Unchanged(outcome) => {
           if behaviors.application_behavior.verbose {
                println!(
                    "VERBOSE:\t Item Unchanged: {}",
                    outcome
                );
            }
        },
        Changeable::Changed(outcome) => {
            let mut after : PathBuf = path_buf.to_path_buf();
            //remove old filename
            after.pop(); 
            //add on new filename
            after.push(outcome);
            if !behaviors.application_behavior.verbose {
                move_path(&path_buf,&after);
            }
            println!(
                "mv \"{}\" \"{}\"",
                path_buf.to_string_lossy(),
                after.to_string_lossy(),
            );
            
        },
        Changeable::Annihilated() => {
            println!(
                "ERROR: Sanitization rules for {} annihilate all valid characters. Doing Nothing.", 
                &path_buf.to_string_lossy()
            );
        }
    };
}

fn move_path(from: &PathBuf, to:&PathBuf) -> Option<String> {
    let both_directories_exist : bool = from.is_dir() && from.exists() && to.is_dir() && to.exists();
    if both_directories_exist {
        //TODO: both directories exist, should we merge or create a numbered instance
        return Option::Some(String::from(""))
    }
    return Option::None

}

pub enum DirectoryConflict {
    Enumerate,
    Merge,
}

pub struct ConflictBehavior {
    pub directory_conflict : DirectoryConflict,
}

impl Default for ConflictBehavior {
    fn default() -> ConflictBehavior {
        ConflictBehavior {
            directory_conflict : DirectoryConflict::Merge
        }
    }
}

pub struct CharacterBehavior {
    pub white_list : Vec<char>,
    pub black_list : Vec<char>,
    pub replacement : char,
    pub replacables : Vec<char>,
    pub cant_enders : Vec<char>,


}

impl Default for CharacterBehavior {
    fn default() -> CharacterBehavior {
        CharacterBehavior {
            white_list : vec!
            [
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
            ],
            black_list : vec!
            [
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
            ],
            replacement : '_',
            replacables : vec!
            [
                //' ', //todo, make up some options switch to replace _ to with space, and visa vera
                // TODO: or allow more CLI input for addition to replacers
                '(',
                ')'
            ],
            cant_enders : vec!
            [
                '.',
                ' '
            ],
        }
    }
}

pub struct ApplicationBehavior {
    pub dry_run : bool,
    pub verbose : bool,
}

impl Default for ApplicationBehavior {
    fn default() -> ApplicationBehavior {
        ApplicationBehavior {
            dry_run : true,
            verbose : true
        }
    }
}

pub struct Behaviors {
    pub character_behavior : CharacterBehavior,
    pub conflict_behavior : ConflictBehavior,
    pub application_behavior : ApplicationBehavior,
}



impl Default for Behaviors {
    fn default() -> Behaviors {
        Behaviors {
            conflict_behavior : ConflictBehavior::default(),
            character_behavior : CharacterBehavior::default(),
            application_behavior : ApplicationBehavior::default(),
        }
    }
}


//one part of a path, not the whole path
fn strip_unwanted(input : &str, behaviors : &Behaviors ) -> Changeable {
    let mut buffer = String::with_capacity(input.len());
    let mut is_dirty : bool = false;

    for c in input.chars(){
        if behaviors.character_behavior.black_list.contains(&c)  {
            //found illegal character, omit it
            is_dirty = true;
        } else if behaviors.character_behavior.replacables.contains(&c){
            //found replacer character, using its replacement
            is_dirty = true;
            buffer.push(behaviors.character_behavior.replacement);
        } else if behaviors.character_behavior.white_list.contains(&c){
            //keep
            buffer.push(c); 
        } else {
            //found non whitelisted char, omit it (aka replace with empty?)
            is_dirty = true;
        }
    }

    //prevent last character in filename (windows restriction)
    //Windows file rules say can't end in space or dot
    while let Some(x) = buffer.chars().last() {
        if behaviors.character_behavior.cant_enders.contains(&x) {
            is_dirty = true;
            buffer.pop();
        }
    }

    
    if is_dirty {
        return Changeable::Changed(buffer)
    } else {
        return Changeable::Unchanged(buffer)
    }
}

pub enum Changeable {
    Changed(String),
    Unchanged(String),
    Annihilated(),
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
    //let dir = base_path;
    
    //if dir.is_dir() {
    //    //for entry in fs::read_dir(dir).unwrap() {
    //    let mut paths: Vec<_> = fs::read_dir(dir).unwrap().map(|red| red.unwrap().path()).collect();
    //    paths.sort();
    //    for path in paths {
    //        //let path = entry.unwrap().path();
    //        if path.is_dir() {
    //            println!("is dir: {}",path.display());
    //        } else {
     //           println!("is file: {}",path.display());
    //        }
  //  
   //     }
    //}
}
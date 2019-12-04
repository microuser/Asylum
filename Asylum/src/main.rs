use std::fs::{self};
use std::path::PathBuf;

fn visit_dirs_sorted(dir: &PathBuf, callback: &dyn Fn(&PathBuf), behaviors : &Behaviors)  {
    if dir.is_dir() {
        //let mut dir_entries : Vec<PathBuf> = ;
        let mut entries : Vec<PathBuf> = fs::read_dir(dir)
            .expect("cannot read directory")
            .filter(Result::is_ok)
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();

        if behaviors.application_behavior.verbose { println!("We found ({}) items in directory: {}", entries.len(),dir.display()); }
        for entry in entries {
            if entry.is_dir() {
                if behaviors.application_behavior.verbose { println!("Diving deeper into: {}", entry.display());} 
                visit_dirs_sorted(&entry, callback, behaviors);
                callback(&entry);
            } else {
                if behaviors.application_behavior.verbose { println!("to analyse {}: {}", if entry.is_dir() {"folder"} else {"file"}  , entry.display());}
                callback(&entry);
            }
        }
    } else {
        println!("The directory does not exist: {}" , dir.display());
    }
}

fn main() {
    // println!("\x1B[25m White");
    // println!("\x1B[26m White");
    // println!("\x1B[27m White");
    // println!("\x1B[28m White");
    // println!("\x1B[29m White");
    // println!("\x1B[30m Black");
    // println!("\x1B[31m Red");
    // println!("\x1B[32m Green");
    // println!("\x1B[33m Yellow");
    // println!("\x1B[34m Blue");
    // println!("\x1B[35m Purple");
    // println!("\x1B[36m Cyan");
    // println!("\x1B[37m White");
    // println!("\x1B[38m White");
    // println!("\x1B[39m White");
    // println!("\x1B[40m White on Black");
    // println!("\x1B[41m White on Red");
    // println!("\x1B[42m White on Green");
    //todo use that cli crate
    let path = std::env::args().nth(1);
    let path = match path {
        Some(x) => { x}
        //todo remove this was for debugging
        None => {String::from("/home/user/a")}
    };
    let path = std::path::Path::new(&path);
    let behaviors = Behaviors::default();

    visit_dirs_sorted(
        &path.to_path_buf(), 
        &|file_or_dir| {
            if behaviors.application_behavior.verbose { println!("Running Callback for: {:?}",file_or_dir);}
            strip_unwated(file_or_dir, &behaviors);
        },
        &behaviors
    );

}

fn strip_unwated(path_buf : &PathBuf, behaviors: &Behaviors) {

    if behaviors.application_behavior.verbose { println!("strip_unwanted: {}" , path_buf.display()); }
    let input = path_buf
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
                move_path(&path_buf,&after,&behaviors);
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

pub fn move_path(from: &PathBuf, to:&PathBuf, behavior: &Behaviors)  {
    let from_exist : bool = from.exists();
    let to_exist : bool = to.exists();
    
    if from_exist && to_exist {
        match behavior.conflict_behavior.directory_conflict {
            DirectoryConflict::Enumerate => move_path_dir_to_dir_enumerate(from,to, behavior),
            DirectoryConflict::Merge =>  move_path_dir_to_dir_merge(from,to, behavior),       
        }
    } else if  from_exist && !to_exist {
        move_path_rename(from, to, behavior);
    } else {
        println!("Error, unable to find source: {}", from.display());
    }
}

fn move_path_rename(from: &PathBuf, to: &PathBuf, behaviors: &Behaviors){
    if behaviors.application_behavior.dry_run {
        if behaviors.application_behavior.verbose {println!("DRYRUN: renamed: '{}' to '{}'", from.display(), to.display())};
    } else {
        match fs::rename(from, to) {
            Ok(_) =>  println!("Renamed: '{}' to '{}'",from.display(), to.display()),
            Err(_) =>  println!("ERROR unable to rename: '{}' to '{}'",from.display(),to.display()),
        }
    }

}

trait StringEnumerated{
    fn trim_enumerate_folder(&self, behavior: &Behaviors) -> String;
    fn trim_enumerate_file(&self, behavior: &Behaviors) -> String;
}

impl StringEnumerated for String{
    fn trim_enumerate_folder(&self, behavior: &Behaviors) -> String{
        if self.len() <= 4 {
            return String::from(self)
        }
        
        let mut chars = self.chars().rev();       
        let matches_pattern : bool = 
            chars.next().unwrap_or(' ').is_numeric() 
            && chars.next().unwrap_or(' ').is_numeric() 
            && chars.next().unwrap_or(' ').is_numeric() 
            && ( chars.next().unwrap_or(' ') == behavior.conflict_behavior.enumerate_folder_character )
            ;

        if matches_pattern {
            String::from(&self[..self.len()-4])
        } else {
            String::from(self)
        }

    }
    fn trim_enumerate_file(&self, behavior: &Behaviors) -> String{
        if self.len() <= 4 {
            return String::from(self)
        }
        
        let mut chars = self.chars().rev();       
        let matches_pattern : bool = 
            chars.next().unwrap_or(' ').is_numeric() 
            && chars.next().unwrap_or(' ').is_numeric() 
            && chars.next().unwrap_or(' ').is_numeric() 
            && ( chars.next().unwrap_or(' ') == behavior.conflict_behavior.enumerate_folder_character )
            ;

        if matches_pattern {
            String::from(&self[..self.len()-4])
        } else {
            String::from(self)
        }
    }
}

trait EnumPathBuf{
    fn apply_enumerate_rules(&self, behavior : &Behaviors) -> PathBuf;
}

impl EnumPathBuf for PathBuf{
    fn apply_enumerate_rules(&self, behavior : &Behaviors) -> PathBuf{
        let mut path_buf = self.to_owned();
        let i : usize = 1;
        loop { 
            if ! path_buf.exists() { 
                break;
            }

            path_buf.set_file_name(
                if path_buf.is_dir() {
                    format!(
                        "{}{}{:03}",
                        path_buf.file_name().expect("expected path for folder").to_string_lossy().to_string().trim_enumerate_folder(behavior),
                        behavior.conflict_behavior.enumerate_folder_character,
                        i,    
                    )
                } else {
                    format!("{}{}{:03}.{}",
                        path_buf.file_stem().expect("expected path for file").to_string_lossy().to_string().trim_enumerate_file(behavior),
                        behavior.conflict_behavior.enumerate_file_character,
                        i,
                        path_buf.extension().unwrap_or_default().to_string_lossy()
                    )
                }
            );
        
        };
        path_buf
    }
}

// fn apply_enumerate_rules(to : &PathBuf, behavior : &Behaviors) -> PathBuf {
//     let mut path_buf = PathBuf::from(to);
//     if ! to.exists() {
//         return path_buf
//     } else {
//         let i : usize = 1;
//         loop { 
//             path_buf.set_file_name(
//                 if path_buf.is_dir() {
//                     format!(
//                         "{}{}{:03}",
//                         path_buf.file_name().expect("expected path for folder").to_string_lossy(),
//                         behavior.conflict_behavior.enumerate_folder_character,
//                         i,    
//                     )
//                 } else {
//                     format!("{}{}{:03}{}",
//                         path_buf.file_stem().expect("expected path for file").to_string_lossy(),
//                         behavior.conflict_behavior.enumerate_file_character,
//                         i,
//                         path_buf.extension().unwrap_or_default().to_string_lossy()
//                     )
//                 }
//             );
                    
//             if ! path_buf.exists() { break;}
//         }
//     }
//     path_buf

// }

fn move_path_dir_to_dir_enumerate(from: &PathBuf, to:&PathBuf, behavior: &Behaviors) {
    let to = to.apply_enumerate_rules(behavior);
    match fs::rename(&from,&to){
        Ok(()) => { 
            //if behavior.application_behavior.verbose 
            //{ 
                println!("Enumerate Renamed: '{}' to '{}'", from.display(), to.display())
            //};
        }
        Err(x) => { 
            println!("Enumerate was unable to rename: '{}' '{}' : {}", &from.display(), &to.display(),x);
        }
    };
}

fn move_path_dir_to_dir_merge(from: &PathBuf, to:&PathBuf, behavior: &Behaviors)  {
    unimplemented!();
}


pub enum DirectoryConflict {
    Enumerate,
    Merge,
}

pub struct ConflictBehavior {
    pub directory_conflict : DirectoryConflict,
    pub enumerate_folder_character : char,
    pub enumerate_file_character : char
}

impl Default for ConflictBehavior {
    fn default() -> ConflictBehavior {
        ConflictBehavior {
            directory_conflict : DirectoryConflict::Enumerate,
            enumerate_folder_character : '_',
            enumerate_file_character : '.'
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
                '=',
                '(',
                ')',
            ],
            replacement : '_',
            replacables : vec!
            [
                //' ', //todo, make up some options switch to replace _ to with space, and visa vera
                // TODO: or allow more CLI input for addition to replacers
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
    pub debug : bool,
}

impl Default for ApplicationBehavior {
    fn default() -> ApplicationBehavior {
        ApplicationBehavior {
            dry_run : false,
            verbose : false,
            debug : false,
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
        } else {
            break;
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
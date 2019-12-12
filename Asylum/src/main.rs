extern crate clap;

use std::fs::{self};
use std::path::PathBuf;
use clap::App;
use clap::Arg;


//use clap::SubCommand;

fn main() {

    let cli_args : clap::ArgMatches = App::new("Asylum")
    .version("1.0")
    .author("Microuser <microuser@users.noreply.github.com>")
    .about("Sanitizes files and folders names")
    .arg(Arg::with_name("path")
        .long("path")
        .short("p")
        .help("path of files or folders to clean")
        //.required(true)
        .index(1)
        .multiple(true)
    )
    .arg(Arg::with_name("dryrun")
        .long("dryrun")
        .short("d")
        .help("Only print actions without touching the disk")
        .multiple(false)
    )
    .arg(Arg::with_name("verbose")
        .help("Sets the verbosity. Use up to two.")
        .long("verbose")
        .short("v")
        .multiple(true)
    )
    .arg(Arg::with_name("profile")
        .help("A profile for groupings of settings for specific situations")
        .long("profile")
        .short("g")
        .multiple(false)
    )
    .arg(Arg::with_name("colors")
        .help("Show from -> to in different colors in terminal std-out")
        .long("colors")
        .short("c")
        )
    .arg(Arg::with_name("no_colors")
        .help("Prevent colors in terminal std-out")
        .long("no-colors")
    )
    .get_matches();

    let behaviors = Behaviors::from_args(&cli_args);

    behaviors.print_verbose(&format!("Starting Asylum with arguments: {:#?}",&cli_args));
    behaviors.print_debug(&format!("Using Application Behaviors: {:#?}",&behaviors));
    
    if cli_args.occurrences_of("path") > 0 {   
        let paths = cli_args.values_of("path").expect("expected content for path").map(|path_string| PathBuf::from(path_string));
        for path in paths {
            visit_dirs_sorted(
                &path.to_path_buf(), 
                &|file_or_dir| {
                    behaviors.print_debug(&format!("Running Callback for: {:?}",file_or_dir));
                    strip_unwanted_file_or_folder(file_or_dir, &behaviors);
                },
                &behaviors
            );  
        }
    } else {
        behaviors.print_error(&format!("Missing path was specified. See --help"));
        std::process::exit(1);
    }

}

fn visit_dirs_sorted(dir: &PathBuf, callback: &dyn Fn(&PathBuf), behaviors : &Behaviors)  {
    if dir.is_dir() {
        //let mut dir_entries : Vec<PathBuf> = ;
        let mut entries : Vec<PathBuf> = fs::read_dir(dir)
            .expect("cannot read directory")
            .filter(Result::is_ok)
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();

        behaviors.print_debug(&format!("We found ({}) items in directory: {}", entries.len(),dir.display()));
        
        for entry in entries {
            if entry.is_dir() {
                behaviors.print_debug(&format!("Diving deeper into: {}", entry.display())); 
                visit_dirs_sorted(&entry, callback, behaviors);
                callback(&entry);
            } else {
                behaviors.print_debug(&format!("to analyse {}: {}", if entry.is_dir() {"folder"} else {"file"}  , entry.display()));
                callback(&entry);
            }
        }
    } else {
        behaviors.print_error(&format!("The directory does not exist: {}" , dir.display()));
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
#[derive(Clone)]
pub enum FileSystemNode{
    File(String),
    Folder(String),
    Other(String),
}

impl std::cmp::PartialEq for FileSystemNode {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (&FileSystemNode::File(a), &FileSystemNode::File(b)) => a == b,
            (&FileSystemNode::Folder(a), &FileSystemNode::Folder(b)) => a == b,
            (&FileSystemNode::Other(a), &FileSystemNode::Other(b)) => a == b,
            _ => false,
        }
    }
}

impl FileSystemNode{
    
    fn get_file_stem(path_buf : &PathBuf) -> FileSystemNode {
        if path_buf.is_dir() {
            FileSystemNode::Folder(
                path_buf
                //just this folder (no parents)
                .file_name()
                .expect("path was expected")
                .to_string_lossy()
                .into_owned()
            )
        } else if path_buf.is_file(){
            //just this file with no extension (no parents)
            FileSystemNode::File(
                path_buf
                .file_stem()
                .expect("path was expected")
                .to_string_lossy()
                .into_owned()
            )
        } else {
            FileSystemNode::Other(
                path_buf
                .file_name()
                .expect("path was expected")
                .to_string_lossy()
                .into_owned()
            )
        }
    }
    
    fn strip_enumeration(self : FileSystemNode, behaviors : &Behaviors) -> FileSystemNode{
        fn string_popper(haystack : &String, enumerate_character : char) -> String{
            let mut has_seen_only_numeric = false;
            let mut chars_to_delete = 0;
            let mut is_valid_sequence = false;
            let mut reverse_chars = haystack.chars().rev();
            //let mut chars_to_remove : usize = 0;
            loop {
                //pop starts from the end
                match reverse_chars.next() {
                    Some(c) => {
                        if c.is_numeric(){
                            has_seen_only_numeric = true;
                            chars_to_delete += 1;
                            continue
                        } else if c == enumerate_character {
                            if has_seen_only_numeric {
                                //it isn't part of the special sequence, it is just a random underscore (enumerate char)
                                is_valid_sequence = true;
                                chars_to_delete += 1;
                            } else {
                                is_valid_sequence = false;
                                chars_to_delete = 0;
                            }
                            break;
                        } else {
                            //if we dont find the perfect sequence by finding an unexpected character, then it is not the right type of ending, 
                            //we reset chars to remove, so that the slice later will be the whole string
                            is_valid_sequence = false;
                            chars_to_delete = 0;
                            break;
                        }
                    },
                    None => {
                        break;
                    }
                };
            };
            if is_valid_sequence {
                String::from(&haystack[..haystack.len()-chars_to_delete])
            } else {
                String::from(haystack)
            }
            
        };
        
        //Question, it seems i have an issue of needing mutable, but yet returning a newly created. Why can't i just mutable inplace?
        return match self {
            FileSystemNode::File(x) => FileSystemNode::File(string_popper(&x, behaviors.conflict_behavior.enumerate_file_character)),
            FileSystemNode::Folder(x) =>  FileSystemNode::Folder(string_popper(&x, behaviors.conflict_behavior.enumerate_folder_character)),
            FileSystemNode::Other(x) => FileSystemNode::Other(string_popper(&x, behaviors.conflict_behavior.enumerate_file_character)),
        }
        
    }

    fn unwrap(self : &FileSystemNode) -> &str{
        match self {
            FileSystemNode::File(x) => x,
            FileSystemNode::Folder(x) => x,
            FileSystemNode::Other(x) => x,
        }
    }
}

fn append_original_extension_if_file(from : &PathBuf, to_striped : &str) -> String {
    if from.is_file(){
        let extension = &from.extension().unwrap_or_default();
        let period_extension = if extension.is_empty() {
            "".to_string()
        } else {
            ".".to_string() + &extension.to_string_lossy()
        };
        //outcome + &period_extension
        [to_striped, &period_extension].concat()
    } else {
        String::from(to_striped)
    }
}


fn strip_unwanted_file_or_folder(path_buf : &PathBuf, behaviors: &Behaviors) {
    behaviors.print_debug(&format!("strip_unwanted: {}" , path_buf.display())); 

    let node : FileSystemNode  = FileSystemNode::get_file_stem(path_buf);
    let potential_striped = node.clone().strip_enumeration(&behaviors);
    //let potential_striped = potential_striped.strip_enumeration(&behaviors);
    
    //question this really looks like the wrong way to write it. I guess i'm just getting around being mutable
    let (node, is_dirty) = if &potential_striped == &node {
        (node, false)
    } else {
        (potential_striped, true)
    };
    
    match strip_unwanted(node.unwrap(), is_dirty, &behaviors) {
        Changeable::Unchanged(_) => {
            behaviors.print_debug(&format!("Item unchanged: {}", path_buf.display()));
        },
        Changeable::Changed(outcome) => {
            let mut after : PathBuf = path_buf.to_path_buf();
            //remove old filename
            after.pop(); 
            //add on new filename
            after.push(append_original_extension_if_file(path_buf,&outcome));

            if behaviors.application_behavior.dryrun {
                behaviors.print_command("mv",&path_buf.to_string_lossy(),&after.to_string_lossy());           
            } else {
                move_path(&path_buf,&after,&behaviors);
            }
            
        },
        Changeable::Annihilated() => {
            behaviors.print_error(&format!(
                "ERROR: Sanitization rules for {} annihilate all valid characters. Doing Nothing.", 
                &path_buf.to_string_lossy()
            )
            );
        }
    };
}

pub fn move_path(from: &PathBuf, to:&PathBuf, behavior: &Behaviors)  {
    let from_exist : bool = from.exists();
    let to_exist : bool = to.exists();
    //it is expected that it was already analysed that the filename actually did change,
    //this if the two exists, it is a collision
    if from_exist && to_exist {
        match behavior.conflict_behavior.directory_conflict {
            DirectoryConflict::Enumerate => move_path_dir_to_dir_enumerate(from,to, behavior),
            DirectoryConflict::Merge =>  move_path_dir_to_dir_merge(from,to, behavior),       
        }
    } else if  from_exist && !to_exist {
        move_path_rename(from, to, behavior);
    } else {
        behavior.print_error(&format!("Error, unable to find source: {}", from.display()));
    }
}

fn move_path_rename(from: &PathBuf, to: &PathBuf, behaviors: &Behaviors){
    if behaviors.application_behavior.dryrun {
        if behaviors.application_behavior.verbose { behaviors.print_command("mv",&from.to_string_lossy(), &to.to_string_lossy()); };
    } else {
        match fs::rename(from, to) {
            Ok(_) =>  behaviors.print_command("mv",&from.to_string_lossy(), &to.to_string_lossy()),
            Err(_) =>  behaviors.print_error(&format!("ERROR unable to rename: '{}' to '{}'",from.display(),to.display())),
        }
    }

}

trait EnumPathBuf{
    fn apply_enumerate_rules(&self, from: &PathBuf, behavior : &Behaviors) -> PathBuf;
}

impl EnumPathBuf for PathBuf{
    //this counts down the enumerate to the lowest number

    fn apply_enumerate_rules(&self, from : &PathBuf , behavior : &Behaviors) -> PathBuf{       
        //looks like we need enumeration
        let mut path_buf = self.to_owned();
        let mut i : usize = 1;
        loop { 
            if ! path_buf.exists() { 
                break;
            }
            //if it is itself, then retain our previous number
            //this prevents thrashing/alternating between two numbers
            //we need to know more, we need to know the from 
            if from == &path_buf {
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
            i+=1;
        };
        path_buf
    }
}

fn move_path_dir_to_dir_enumerate(from: &PathBuf, to:&PathBuf, behaviors: &Behaviors) {
    let to = to.apply_enumerate_rules(from, behaviors);
    if from != &to {
        match fs::rename(&from,&to){
            Ok(()) => { 
                behaviors.print_verbose(&format!("Enumerate Renamed: '{}' to '{}'", from.to_string_lossy(), to.to_string_lossy()));
            }
            Err(x) => { 
                behaviors.print_error(&format!("Enumerate was unable to rename: '{}' '{}' : {}", &from.display(), &to.display(),x));
            }
        };
    } else {
        behaviors.print_debug(&format!("Enumerate retained the name : '{}'", from.to_string_lossy()));
    }

}

fn move_path_dir_to_dir_merge(from: &PathBuf, to:&PathBuf, behaviors: &Behaviors)  {
    println!("From: {}", &from.display());
    println!("To: {}", &to.display());
    println!("Behaviors: {:?}", behaviors.conflict_behavior.directory_conflict);
    behaviors.print_error("Unimplemented Merge");
    unimplemented!();
}

#[derive(Debug)]
pub enum DirectoryConflict {
    Enumerate,
    Merge,
}

#[derive(Debug)]
pub struct TerminalBehavior {
    pub colors : bool,
    pub color_from : String,
    pub color_to : String,
    pub color_default : String,
    pub color_verbose : String,
    pub color_debug : String,
    pub color_error : String,
    pub spacer : String,
    pub quoter : String,
}

impl TerminalBehavior {
    fn from_args(_args : &clap::ArgMatches) -> TerminalBehavior {
        let terminal_behavior = TerminalBehavior::default();
        //todo process args here
        terminal_behavior
    }
}

impl Default for TerminalBehavior{
    fn default() -> TerminalBehavior {
        TerminalBehavior {
            colors : true,
            color_from : "\x1B[33m".to_string(),
            color_to : "\x1B[32m".to_string(),
            color_default : "\x1B[37m".to_string(),
            color_verbose : "\x1B[35m".to_string(),
            color_debug : "\x1B[36m".to_string(),
            color_error : "\x1B[31m".to_string(),
            spacer : "   ".to_string(),
            quoter : "'".to_string(),
        }

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
    }
}

#[derive(Debug)]
pub struct ConflictBehavior {
    pub directory_conflict : DirectoryConflict,
    pub enumerate_folder_character : char,
    pub enumerate_file_character : char
}

impl ConflictBehavior {
    fn from_args(_args : &clap::ArgMatches) -> ConflictBehavior {
        let conflict_behavior = ConflictBehavior::default();
        //todo here is where we insert allow difference between merge and enumerate
        return conflict_behavior
    }
}



impl Default for ConflictBehavior {
    fn default() -> ConflictBehavior {
        ConflictBehavior {
            directory_conflict : DirectoryConflict::Enumerate,
            enumerate_folder_character : '_', //TODO FIX to be _
            enumerate_file_character : '_'
        }
    }
}

#[derive(Debug)]
pub struct CharacterBehavior {
    pub cant_beginners : Vec<char>,
    pub white_list : Vec<char>,
    pub black_list : Vec<char>,
    //pub replacement : char,
    //pub replacables : Vec<char>,
    pub cant_enders : Vec<char>,
    pub replacer_strings : Vec<(String,String)>,
    pub replacer_chars : Vec<(char,char)>,
}

impl CharacterBehavior {
    fn from_args(_args : &clap::ArgMatches ) -> CharacterBehavior{
        let character_behavior = CharacterBehavior::default();
        //here is where you insert into the vectors for different behavior.character_behavior
        //todo solve logic of which we can remove from default list
        //todo allow some general group names
        return character_behavior
    }
}

impl Default for CharacterBehavior {
    fn default() -> CharacterBehavior {
        CharacterBehavior {
            //cant beginners mean that a filename can't begin with this, 
            //this character must be in the whitelist
            //it also means that when multiple of them are reached, they collapse into 1
            //they are also equivilant to eachother, so " 1 __ " becomres "1 " while " 1_  _" becomes "1_"
            cant_beginners : vec!
            [
                ' ','_'
            ],
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
                //'_',
                ' ',
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
                //'-',
                '`',
                '#',
                '~',
                '^',
                '+',
                '=',
                '(',
                ')',
            ],
            //replacement : ' ',
            //replacables : vec!
            //[
            //    '.'
            //    //' ', //todo, make up some options switch to replace _ to with space, and visa vera
            //    // TODO: or allow more CLI input for addition to replacers
            //],
            replacer_chars : vec![
                //space equivilants
                ('(',' '),
                (')',' '),
                ('<',' '),
                ('>',' '),
                ('=',' '),
                ('[',' '),
                (']',' '),
                ('-',' '),
                ('.',' '),
                (',',' '),
                ('–',' '), //en-dash
                ('—',' '), //em-dash
                ('_',' '), //underscore
                ('_',' '), //lookalike underscope

                //grave
                ('.',' '),
                ('À','A'),
                ('à','a'),
                ('È','E'),
                ('è','e'),
                ('Ì','I'),
                ('ì','i'),
                ('Ò','O'),
                ('ò','o'),
                ('Ù','U'),
                ('ù','u'),
                //Acute
                ('Á','A'),
                ('á','a'),
                ('É','E'),
                ('é','e'),
                ('Í','I'),
                ('í','i'),
                ('Ó','O'),
                ('ó','o'),
                ('Ú','U'),
                ('ú','u'),
                ('Ý','Y'),
                ('ý','y'),
                //Circumflex
                ('Â','A'),
                ('â','a'),
                ('Ê','E'),
                ('ê','e'),
                ('Î','I'),
                ('î','i'),
                ('Ô','O'),
                ('ô','o'),
                ('Û','U'),
                ('û','u'),
                //telde
                ('Ã','A'),
                ('ã','a'),
                ('Ñ','N'),
                ('ñ','n'),
                ('Õ','o'),
                ('õ','o'),
                //Umlaut
                ('Ä','A'),
                ('ä','a'),
                ('Ë','E'),
                ('ë','e'),
                ('Ï','I'),
                ('ï','i'),
                ('Ö','O'),
                ('ö','o'),
                ('Ü','U'),
                ('ü','u'),
                ('Ÿ','Y'),
                ('ÿ','y'),
                //Czech
                ('Š','S'),
                ('š','s'),
                ('Ž','Z'),
                ('ž','z'),
                //Nordic
                ('Ø','O'),
                ('ø','o'),
                ('Å','å'),


                //math
                ('µ','u'),
                //symbols




                
            ],
            replacer_strings : vec![
                (" - ".to_string()," ".to_string()),
                ("Æ".to_string(),"AE".to_string()),
                //remove dates
                ("(1950)".to_string(),"".to_string()),
                ("(1951)".to_string(),"".to_string()),
                ("(1952)".to_string(),"".to_string()),
                ("(1953)".to_string(),"".to_string()),
                ("(1954)".to_string(),"".to_string()),
                ("(1955)".to_string(),"".to_string()),
                ("(1956)".to_string(),"".to_string()),
                ("(1957)".to_string(),"".to_string()),
                ("(1958)".to_string(),"".to_string()),
                ("(1959)".to_string(),"".to_string()),
                ("(1960)".to_string(),"".to_string()),
                ("(1961)".to_string(),"".to_string()),
                ("(1962)".to_string(),"".to_string()),
                ("(1963)".to_string(),"".to_string()),
                ("(1964)".to_string(),"".to_string()),
                ("(1965)".to_string(),"".to_string()),
                ("(1966)".to_string(),"".to_string()),
                ("(1967)".to_string(),"".to_string()),
                ("(1968)".to_string(),"".to_string()),
                ("(1969)".to_string(),"".to_string()),
                ("(1970)".to_string(),"".to_string()),
                ("(1971)".to_string(),"".to_string()),
                ("(1972)".to_string(),"".to_string()),
                ("(1973)".to_string(),"".to_string()),
                ("(1974)".to_string(),"".to_string()),
                ("(1975)".to_string(),"".to_string()),
                ("(1976)".to_string(),"".to_string()),
                ("(1977)".to_string(),"".to_string()),
                ("(1978)".to_string(),"".to_string()),
                ("(1979)".to_string(),"".to_string()),
                ("(1980)".to_string(),"".to_string()),
                ("(1981)".to_string(),"".to_string()),
                ("(1982)".to_string(),"".to_string()),
                ("(1983)".to_string(),"".to_string()),
                ("(1984)".to_string(),"".to_string()),
                ("(1985)".to_string(),"".to_string()),
                ("(1986)".to_string(),"".to_string()),
                ("(1987)".to_string(),"".to_string()),
                ("(1988)".to_string(),"".to_string()),
                ("(1989)".to_string(),"".to_string()),
                ("(1990)".to_string(),"".to_string()),
                ("(1991)".to_string(),"".to_string()),
                ("(1992)".to_string(),"".to_string()),
                ("(1993)".to_string(),"".to_string()),
                ("(1994)".to_string(),"".to_string()),
                ("(1995)".to_string(),"".to_string()),
                ("(1996)".to_string(),"".to_string()),
                ("(1997)".to_string(),"".to_string()),
                ("(1998)".to_string(),"".to_string()),
                ("(1999)".to_string(),"".to_string()),
                ("(2000)".to_string(),"".to_string()),
                ("(2001)".to_string(),"".to_string()),
                ("(2002)".to_string(),"".to_string()),
                ("(2003)".to_string(),"".to_string()),
                ("(2004)".to_string(),"".to_string()),
                ("(2005)".to_string(),"".to_string()),
                ("(2006)".to_string(),"".to_string()),
                ("(2007)".to_string(),"".to_string()),
                ("(2008)".to_string(),"".to_string()),
                ("(2009)".to_string(),"".to_string()),
                ("(2010)".to_string(),"".to_string()),
                ("(2011)".to_string(),"".to_string()),
                ("(2012)".to_string(),"".to_string()),
                ("(2013)".to_string(),"".to_string()),
                ("(2014)".to_string(),"".to_string()),
                ("(2015)".to_string(),"".to_string()),
                ("(2016)".to_string(),"".to_string()),
                ("(2017)".to_string(),"".to_string()),
                ("(2018)".to_string(),"".to_string()),
                ("(2019)".to_string(),"".to_string()),
                ("(2020)".to_string(),"".to_string()),
                ("(2021)".to_string(),"".to_string()),
                ("(2022)".to_string(),"".to_string()),
                ("(2023)".to_string(),"".to_string()),
                ("(2024)".to_string(),"".to_string()),
                ("(2025)".to_string(),"".to_string()),
                ("(2026)".to_string(),"".to_string()),
                ("(2027)".to_string(),"".to_string()),
                ("(2028)".to_string(),"".to_string()),
                ("(2029)".to_string(),"".to_string()),
                //ownership tags
                (".DVDScr".to_string(), "".to_string()),
                ("BRRip".to_string(), "".to_string()),
                ("Release-Lounge".to_string(), "".to_string()),
                ("XviD-MAXSPEED".to_string(), "".to_string()),
                ("www.torentz.3xforum.ro.avi".to_string(), "".to_string()),
                ("DRONES[EtHD]".to_string(), "".to_string()),
                ("Addiction10".to_string(), "".to_string()),
                ("-IZON-".to_string(), "".to_string()),
                ("(Kingdom Release)".to_string(), "".to_string()),
                ("rocknonstop".to_string(), "".to_string()),
                ("(DVDRiP)".to_string(), "".to_string()),
                ("(XviD)".to_string(), "".to_string()),
                ("Felony".to_string(), "".to_string()),
                ("Felony".to_string(), "".to_string()),
                
                ("FraMeSToR".to_string(), "".to_string()),
                ("WEBRiP".to_string(), "".to_string()),
                ("-LEGi0N".to_string(), "".to_string()),
                ("WEB-DL".to_string(), "".to_string()),
                (".DD5.".to_string(), "".to_string()),
                ("H264".to_string(), "".to_string()),
                ("-FGT".to_string(), "".to_string()),
                (".dvdrip".to_string(), "".to_string()),
                (".BRRip".to_string(), "".to_string()),
                (".goliath".to_string(), "".to_string()),
                ("-PLAYNOW".to_string(), "".to_string()),
                ("-ESiR".to_string(), "".to_string()),
                (" REPACK".to_string(), "".to_string()),
                ("-bipolar".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),
                
                //video formats
                ("H264".to_string(), "".to_string()),
                ("360p".to_string(), "".to_string()),
                ("stereo".to_string(), "".to_string()),
                (".720.".to_string(), "".to_string()),
                ("x264".to_string(), "".to_string()),
                ("THORA".to_string(), "".to_string()),
                ("H.264".to_string(), "".to_string()),
                ("MP4-KTR[rarbg]".to_string(), "".to_string()),
                ("XviD".to_string(), "".to_string()),
                (".720.".to_string(), "".to_string()),
                ("XviD-SUMO".to_string(), "".to_string()),
                ("-SUMO".to_string(), "".to_string()),
                ("Blueray".to_string(), "".to_string()),
                ("1080p".to_string(), "".to_string()),
                ("DTS-HD".to_string(), "".to_string()),
                ("x264".to_string(), "".to_string()),
                ("-Grym".to_string(), "".to_string()),
                ("AC3".to_string(), "".to_string()),
                ("720p".to_string(), "".to_string()),
                ("HEVC".to_string(), "".to_string()),
                ("x265".to_string(), "".to_string()),
                ("X264".to_string(), "".to_string()),
                ("720p".to_string(), "".to_string()),
                ("DVDRip".to_string(), "".to_string()),
                ("www.torentz.3xforum.ro".to_string(), "".to_string()),
                ("-MAXSPEED".to_string(), "".to_string()),
                ("BluRay".to_string(), "".to_string()),
                ("-DRONES".to_string(), "".to_string()),
                ("-drones".to_string(), "".to_string()),
                ("DRONES[EtHD]".to_string(), "".to_string()),
                ("-EVO".to_string(), "".to_string()),
                ("-aAF".to_string(), "".to_string()),
                ("HDTV".to_string(), "".to_string()),
                ("hdtv".to_string(), "".to_string()),
                ("BDRip".to_string(), "".to_string()),
                ("HDRip".to_string(), "".to_string()),
                ("-ROVERS".to_string(), "".to_string()),
                ("-rovers".to_string(), "".to_string()),
                ("DivX5".to_string(), "".to_string()),
                ("-Chedda".to_string(), "".to_string()),
                ("bluray".to_string(), "".to_string()),
                ("-usury".to_string(), "".to_string()),
                ("-CtrlHD".to_string(), "".to_string()),
                ("XVID".to_string(), "".to_string()),
                ("DVDSCR".to_string(), "".to_string()),
                ("TRiPS".to_string(), "".to_string()),
                ("-w4f".to_string(), "".to_string()),
                ("-AN0NYM0US".to_string(), "".to_string()),
                ("-DEFLATE".to_string(), "".to_string()),
                ("-RARBG".to_string(), "".to_string()),
                ("-PSYCHD".to_string(), "".to_string()),
                ("-RARBG".to_string(), "".to_string()),
                ("-GECKOS".to_string(), "".to_string()),
                ("-SPARKS".to_string(), "".to_string()),
                ("-DON".to_string(), "".to_string()),
                (".DTS.".to_string(), "".to_string()),
                (" DTS ".to_string(), "".to_string()),
                ("-DTS-".to_string(), "".to_string()),
                ("(SiRiUs sHaRe)".to_string(), "".to_string()),
                (".DD+5.1.".to_string(), "".to_string()),
                ("-SiGMA".to_string(), "".to_string()),
                (".AMZN.".to_string(), "".to_string()),
                (".SikSyko".to_string(), "".to_string()),
                ("-ViP3R".to_string(), "".to_string()),
                ("-Masta[ETRG]".to_string(), "".to_string()),
                ("-deadpool".to_string(), "".to_string()),
                ("DD5.1".to_string(), "".to_string()),
                ("NTSC".to_string(), "".to_string()),
                ("DVD".to_string(), "".to_string()),
                ("-TheRival".to_string(), "".to_string()),
                (" ETRG".to_string(), "".to_string()),
                ("-DRONES".to_string(), "".to_string()),
                ("BRRip".to_string(), "".to_string()),
                ("-x0r".to_string(), "".to_string()),
                ("AAC2".to_string(), "".to_string()),
                ("-SADPANDA".to_string(), "".to_string()),
                (".PROPER.".to_string(), "".to_string()),
                ("-FraMeSToR".to_string(), "".to_string()),
                (".REMUX".to_string(), "".to_string()),
                (".DTS-HD.".to_string(), "".to_string()),
                (".MA.".to_string(), "".to_string()),
                (".5.1.".to_string(), "".to_string()),
                (".AVC.".to_string(), "".to_string()),
                ("-AMIABLE".to_string(), "".to_string()),
                ("Bluray".to_string(), "".to_string()),
                ("-BATV".to_string(), "".to_string()),
                ("-batv".to_string(), "".to_string()),
                ("-sparks".to_string(), "".to_string()),
                ("IMAX".to_string(), "".to_string()),
                ("HDCLUB".to_string(), "".to_string()),
                ("iMBT".to_string(), "".to_string()),
                ("FLEET".to_string(), "".to_string()),
                ("ROFL".to_string(), "".to_string()),
                ("h264".to_string(), "".to_string()),
                (" WEB ".to_string(), "".to_string()),
                ("USURY".to_string(), "".to_string()),
                ("WEBRip".to_string(), "".to_string()),
                ("LIMITED".to_string(), "".to_string()),
                ("-limited".to_string(), "".to_string()),
                ("-FANDANGO".to_string(), "".to_string()),
                ("-BitMeTV".to_string(), "".to_string()),
                ("-DEADPOOL".to_string(), "".to_string()),
                ("-FooKaS".to_string(), "".to_string()),
                ("-SPARKS".to_string(), "".to_string()),
                ("-Absinth".to_string(), "".to_string()),
                ("-CM8".to_string(), "".to_string()),
                ("-BLOW".to_string(), "".to_string()),
                ("-REFiNED".to_string(), "".to_string()),
                ("-CRiSC".to_string(), "".to_string()),
                (".xvid".to_string(), "".to_string()),
                (".REMASTERED.".to_string(), "".to_string()),
                ("DvDrip".to_string(), "".to_string()),
                (".STUDIO.AUDIO".to_string(), "".to_string()),
                (".XVID".to_string(), "".to_string()),
                (".AC3-5.1".to_string(), "".to_string()),
                (".HQ.Hive".to_string(), "".to_string()),
                ("-aXXo".to_string(), "".to_string()),
                (" FXG".to_string(), "".to_string()),
                ("-CiNEFiLE".to_string(), "".to_string()),
                (".V2".to_string(), "".to_string()),
                (" aXXo".to_string(), "".to_string()),
                ("480p".to_string(), "".to_string()),
                ("BRRiP".to_string(), "".to_string()),
                ("AAC".to_string(), "".to_string()),
                ("ExtraTorrentRG".to_string(), "".to_string()),
                ("REMASTERED".to_string(), "".to_string()),
                ("HQ Hive".to_string(), "".to_string()),
                ("CiNEFiLE".to_string(), "".to_string()),
                (".DvDRip".to_string(), "".to_string()),
                ("-FxM".to_string(), "".to_string()),
                ("720P".to_string(), "".to_string()),
                ("MP3.MVGroup.org".to_string(), "".to_string()),
                (".XviD".to_string(), "".to_string()),
                (".PDTV".to_string(), "".to_string()),
                ("NTSC".to_string(), "".to_string()),
                ("DD5.1".to_string(), "".to_string()),
                (" MMI".to_string(), "".to_string()),
                
                ("-NYDIC".to_string(), "".to_string()),
                ("-LaUzu".to_string(), "".to_string()),
                ("_8thSin".to_string(), "".to_string()),
                (".hdrip".to_string(), "".to_string()),
                (".HC".to_string(), "".to_string()),
                ("SyMbIoTe".to_string(), "".to_string()),
                ("[www.meWarez.org]".to_string(), "".to_string()),
                ("Cronus".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),
                ("".to_string(), "".to_string()),

                
                
            ],
            cant_enders : vec!
            [
                '.',' '
            ],
        }
    }
}

#[derive(Debug)]
pub struct ApplicationBehavior {
    pub dryrun : bool,
    pub verbose : bool,
    pub debug : bool,
    pub colors : bool,
}
impl ApplicationBehavior {
    fn from_args(args : &clap::ArgMatches ) -> ApplicationBehavior{
        let application_behavior = ApplicationBehavior {
            dryrun : (args.occurrences_of("dryrun") > 0),
            verbose : (args.occurrences_of("verbose") > 0),
            debug : (args.occurrences_of("verbose") > 1 ),
            colors : (args.occurrences_of("colors") > 0 ) && !(args.occurrences_of("no-colors") > 0),
        };
        return application_behavior;
    }
}
impl Default for ApplicationBehavior {
    fn default() -> ApplicationBehavior {
        ApplicationBehavior {
            dryrun : false,
            verbose : false,
            debug : false,
            colors : true
        }
    }
}
#[derive(Debug)]
pub struct Behaviors {
    pub character_behavior : CharacterBehavior,
    pub conflict_behavior : ConflictBehavior,
    pub application_behavior : ApplicationBehavior,
    pub terminal_behavior : TerminalBehavior,
}
impl Behaviors{
    fn print_error(self : &Behaviors, message : &str){
        
        if self.terminal_behavior.colors {
            println!("{}{}{}",&self.terminal_behavior.color_error, message, &self.terminal_behavior.color_default);
        } else {
            println!("{}",message);
        }
        
    }
    fn from_args(args : &clap::ArgMatches) ->  Behaviors {
        return Behaviors {
            application_behavior : ApplicationBehavior::from_args(&args),
            conflict_behavior : ConflictBehavior::from_args(&args),
            character_behavior : CharacterBehavior::from_args(&args),
            terminal_behavior : TerminalBehavior::from_args(&args),
        }
    }
    fn print_verbose(self : &Behaviors, message : &str){
        if self.application_behavior.verbose {
            if self.terminal_behavior.colors {
                println!("{}{}{}",&self.terminal_behavior.color_verbose, message, &self.terminal_behavior.color_default);
            } else {
                println!("{}",message);
            }
        }
    }
    fn print_debug(self : &Behaviors, message : &str){
        if self.application_behavior.debug {
            if self.terminal_behavior.colors {
                println!("{}{}{}",&self.terminal_behavior.color_debug, message, &self.terminal_behavior.color_default);
            } else {
                println!("{}",message);
            }
        } 
    }
    fn print_command(self : &Behaviors, command : &str, from : &str, to : &str){
        if self.terminal_behavior.colors {
            println!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                &self.terminal_behavior.color_default,
                command, 
                &self.terminal_behavior.spacer, 
                &self.terminal_behavior.quoter, 
                &self.terminal_behavior.color_from ,
                from, 
                &self.terminal_behavior.color_default,
                &self.terminal_behavior.quoter, 
                &self.terminal_behavior.spacer,
                &self.terminal_behavior.quoter, 
                &self.terminal_behavior.color_to,
                to,
                &self.terminal_behavior.color_default,
                &self.terminal_behavior.quoter, 
             );
        } else {
            println!("{}{}{}{}{}{}{}{}{}",
                command, 
                &self.terminal_behavior.spacer, 
                &self.terminal_behavior.quoter, 
                from, 
                &self.terminal_behavior.quoter, 
                &self.terminal_behavior.spacer, 
                &self.terminal_behavior.quoter, 
                to, 
                &self.terminal_behavior.quoter
            );
        }
    }

}

impl Default for Behaviors {
    fn default() -> Behaviors {
        Behaviors {
            conflict_behavior : ConflictBehavior::default(),
            character_behavior : CharacterBehavior::default(),
            application_behavior : ApplicationBehavior::default(),
            terminal_behavior : TerminalBehavior::default(),
        }
    }
}


//one part of a path, not the whole path
fn strip_unwanted(input : &str, mut is_dirty : bool, behaviors : &Behaviors ) -> Changeable {
    let mut input = input.to_owned();
    let mut buffer = String::with_capacity(input.len());
    //let mut is_dirty : bool = false;

    let is_hidden : bool = &input[..1] == ".";
    if is_hidden {
        buffer.push('.');
        input = String::from(&input[1..]);
    };

    for replacer in &behaviors.character_behavior.replacer_strings {
        if input.find(&replacer.0).is_some() {
//            let potentialChange = &input.clone();
            let potential_change = input.replace(
                &replacer.0,
                &replacer.1,
            );
            //This prevents blank entries from messing up everything
            if potential_change != input {
                is_dirty = true;
                input = potential_change;
            }
        }
    }
    //whenever you push a character, don't push two spaces in a row
    //this starting condition prevents a space from starting
    let mut last_push_was_cant_beginner : bool = true;
    for mut c in input.chars() {

        //pre-process characters and replace over c if key matches
        for (needle,replacer) in &behaviors.character_behavior.replacer_chars {
            if needle == &c {
                c = *replacer;
                is_dirty = true;
                break;
            }
        }

        if behaviors.character_behavior.black_list.contains(&c) {
            //found illegal character, omit it
            is_dirty = true;
        } else if behaviors.character_behavior.white_list.contains(&c) {
            //keep because it is in white list
            //prevent writing two spaces in a row
            let cant_begin_or_cant_consecutave = behaviors.character_behavior.cant_beginners.contains(&c);
            if last_push_was_cant_beginner && cant_begin_or_cant_consecutave  {
                //mark dirty since we didn't push
                is_dirty = true;
            } else {
                buffer.push(c); 
                last_push_was_cant_beginner = cant_begin_or_cant_consecutave ;                
            }
            
            
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

    //clean up multiple spaces


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
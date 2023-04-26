// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{io::Read, thread, time::{Duration, SystemTime, UNIX_EPOCH, self, Instant}, path::Path, mem};
mod yu;
use filesize::PathExt;
use tauri::{Manager, api::file::read_string, State, Runtime};
// #[tauri::command] // add this attribute
// fn read_and_emit(app_handle: tauri::AppHandle) -> Result<(), String> { // change return type to Result
//   let content = read_string("/home/roger/.config/LogLinktoDisk/links.md").unwrap(); // use ? instead of unwrap
//   app_handle.emit_to(
//     "main",
//     "message", 
//     content
//   )
//   .map_err(|e| e.to_string()) // convert error to string
// }
// // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn loadmarkdown(name: String, window: Window,g:State<FileSizeFinder>) -> String {
    let mut content=String::new();
    let app_handle = window.app_handle();
    let path=PathBuf::from(name.clone());
    let parent=path.clone();

    // let mut file = std::fs::File::open("/home/roger/.config/LogLinktoDisk/links.md").unwrap();
    let mut file = std::fs::File::open(name).unwrap();
    
  
  app_handle.emit_to(
      "main",
      "folder-size",
      {
        sizeunit::size(g.find_size(&path.to_string_lossy()),true)
      },
    )
    .map_err(|e| e.to_string()).unwrap_or(println!("failed to send file size"));
  
  app_handle.emit_to(
      "main",
      "grandparent-loc",
      parent.parent().unwrap().to_string_lossy().to_string(),
    )
    .map_err(|e| e.to_string()).unwrap_or(println!("failed to send grandparentloc"));
  
  app_handle.emit_to(
      "main",
      "parent-loc",
      parent.to_string_lossy().to_string(),
    )
    .map_err(|e| e.to_string()).unwrap_or(println!("failed to send parent loc"));
  
    file.read_to_string(&mut content).unwrap();
    markdown::to_html_with_options(
        &content ,
        &markdown::Options::gfm()
    ).unwrap()
    // format!("Hello, {}! You've been greeted from Rust!", name)
}

// fn main()->Result<(),()> {
    

//     tauri::Builder::default()
//     .setup(|app| {
//                 // get an instance of AppHandle
//                 let app_handle = app.handle();
//                 thread::spawn(move||{
//                 read_and_emit(app_handle).unwrap(); // call the command function
//                 thread::sleep(Duration::from_secs(3));
//             });
//         Ok(())
//     })
//     .invoke_handler(tauri::generate_handler![loadmarkdown])
//     .run(tauri::generate_context!())
//     .expect("error while running tauri application");

//     Ok(())
// }
// src-tauri/src/main.rs

use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle,  Window};
mod fsize;
use fsize::*;
mod sizeunit;

// define a struct to represent a file or directory
#[derive(Serialize,Clone,Debug)]
struct FileItem {
  name: String,
  path: String,
  is_dir: bool,
  size:String,
  rawfs:u64,
  lmdate:String,
  timestamp:i64,
  foldercon:i32
  // grandparent:String,
  // parent:String
}
const CACHE_EXPIRY:u64=60;

// define a command to list the files and directories in a given path
#[tauri::command]
async fn list_files(path: String, window: Window, state: State<'_, FileSizeFinder>) -> Result<serde_json::Value, String> {
  
  
  let now = SystemTime::now();
  
  let duration = now.duration_since(UNIX_EPOCH).unwrap();
  
  let startime = duration.as_secs();
  
  println!("{}----{}",path,startime);

  // get the app handle from the window
 
  let app_handle = window.app_handle();
  app_handle.emit_to(
    "main",
    "start-timer",
    "",
  )
  .map_err(|e| e.to_string())?;

  // convert the path to a PathBuf
  let path = PathBuf::from(path);
let parent=path.clone();
let fcount=fs::read_dir(&path).unwrap().count();
println!("folders---{}",fcount);
app_handle.emit_to(
  "main",
  "folder-count",
  fcount,
)
.map_err(|e| e.to_string())?;
app_handle.emit_to(
  "main",
  "grandparent-loc",
  parent.parent().unwrap().to_string_lossy().to_string(),
)
.map_err(|e| e.to_string())?;

app_handle.emit_to(
  "main",
  "parent-loc",
  parent.to_string_lossy().to_string(),
)
.map_err(|e| e.to_string())?;
let mut tfsize=0;
//    let mut finder = ;
  // check if the path exists and is a directory
  if path.exists() && path.is_dir() {
    // read the entries in the directory
    match fs::read_dir(&path) {
      Ok(entries) => {
        // create an empty vector to store the file items
        let mut files:Vec<FileItem> = Vec::new();
        let mut count=0;
        let mut totsize=0;
        let mut now = Instant::now();
        let update:Vec<u64>=vec![20,40,60,80];

        let mut firsttime=true;
        let mut folderloc=0;
        // loop through the entries
        for entry in entries {
          count+=1;
          let msval=update.iter().next().unwrap_or(&90);
          // get the metadata of the entry
          if let Ok(metadata) = entry.as_ref().unwrap().metadata() {
            // get the name and path of the entry
            let name = entry.as_ref().unwrap().file_name().into_string().unwrap();
            let path = entry.as_ref().unwrap().path().to_string_lossy().into_owned();
            if !entry.as_ref().unwrap().path().is_dir(){
              match(entry.as_ref().unwrap().path().extension()){
                Some(g)=>{
                  if g.to_string_lossy().to_string()=="rs"{
                    folderloc=fs::read_to_string(entry.as_ref().unwrap().path()).expect("Unable to open file").lines().count();
                    println!("{}",folderloc);
                  }

                },
                None=>{

                }
              }
            }
            
            // check if the entry is a file or a directory
            let is_dir = metadata.is_dir();
            // let csizebefore=state.print_cache_size();
            let size=state.find_size(&path);
            let foldercon=state.foldercon(&path);
            let mut tr;
            // let size=0;
            // println!("{}---{}",path,foldercon);
            let (lmdate,timestamp)=lastmodified(&path);
            // create a file item from the entry data
            let file = FileItem { 
                name:name.clone(),
                path:path.clone(),
                is_dir,
                size:{
                 tr=if(size>1){
                    tfsize+=size;
                    // println!("{}",tfsize);
                   sizeunit::size(size,true)
                  }
                  else{
                    "".to_string()
                  };
                  if(folderloc>0){
                    tr.clone() + " (" + &folderloc.to_string() + ")"
                  }
                  else{
                    tr.clone()

                  }
                },
                rawfs:size,    
                lmdate:lmdate.clone(),
                timestamp:timestamp,
                foldercon:foldercon
                // grandparent:parent.parent().unwrap().to_string_lossy().to_string(),
                // parent:parent.to_string_lossy().to_string()
                //tfsize
                // {
                //   let size=FileSizeFinder::new(CACHE_EXPIRY).find_size(&parent.to_string_lossy().to_string());
                //   let tr=if(size>1){
                    
                //    sizeunit::size(size,true)
                //   }
                //   else{
                //     "".to_string()
                //   };
                //   tr
                // }
            };
            // push the file item to the vector
            totsize+=mem::size_of_val(&file);
            // totsize+=mem::size_of_val(&name);
            // totsize+=mem::size_of_val(&path);
            // totsize+=mem::size_of_val(&is_dir);
            // totsize+=mem::size_of_val(&tr);
            // totsize+=mem::size_of_val(&size);
            // totsize+=mem::size_of_val(&lmdate);
            // totsize+=mem::size_of_val(&timestamp);
            // totsize+=mem::size_of_val(&foldercon);
            println!("{}-----{} out of {} \t{}--{}",sizeunit::size(totsize as u64,true),count,fcount,name,path);
            files.push(file);
            let elapsed = now.elapsed();
            
            if elapsed.ge(&Duration::from_millis(*msval)) || firsttime || (count%20==0 && elapsed.ge(&Duration::from_millis(20)))
            {
              firsttime=false;
            //Todo move to a separate function and call after every select interval instead of count%10
              app_handle.emit_to(
                "main",
                "list-files",
                serde_json::to_string(&files.clone()).unwrap(),
              )
              .map_err(|e| e.to_string())?;
            
            app_handle.emit_to(
                "main",
                "folder-size",
                {
                  sizeunit::size(tfsize,true)
                },
              )
              .map_err(|e| e.to_string())?;
            }
          
            now = Instant::now();
          }

        }
        app_handle.emit_to(
          "main",
          "list-files",
          serde_json::to_string(&files.clone()).unwrap(),
        )
        .map_err(|e| e.to_string())?;
      
      app_handle.emit_to(
          "main",
          "folder-size",
          {
            sizeunit::size(tfsize,true)
          },
        )
        .map_err(|e| e.to_string())?;
        // sort the vector by name
        // files.sort_by(|a, b| a.name.cmp(&b.name));
        // emit an event to the frontend with the vector as payload
        println!("reachedhere");
        // println!("{:?}",serde_json::to_string(&files.clone()).unwrap());
        
       state.print_cache_size();
        let now = SystemTime::now();
  
        let duration = now.duration_since(UNIX_EPOCH).unwrap();
        
        let endtime = duration.as_secs();
        
        println!("endtime----{}",endtime-startime);
        
        app_handle.emit_to(
          "main",
          "stop-timer",
          "",
        )
        .map_err(|e| e.to_string())?;

        
        
      // app_handle.emit_to(
      //         "main",
      //         "list-files",
      //         serde_json::to_string(&files.clone()).unwrap(),
      //       )
      //       .map_err(|e| e.to_string())?;
          
          
        // return Ok with the vector
        Ok(serde_json::to_value(&files.clone()).unwrap())
      },
      Err(e) => {
        // return Err with the error message
        Err(e.to_string())
      }
    }
  } else {
    
    // return Err with an invalid path message
    Err("Invalid path".to_string())
  }
  
}
#[tauri::command]
async fn openpath<R: Runtime>(path: String,app: tauri::AppHandle<R>, window: tauri::Window<R>) -> Result<(), String> {
  println!("{}",path);
  match(opener::open(path)){
    Ok(g)=>{
      println!("opening")
      
    },Err(e)=>{
      
      println!("error opening file")
    }
  };
  Ok(())
}
use chrono::{DateTime, Local, Utc};
fn lastmodified(path:&str)->(String,i64){

    // get the metadata of the path
    let metadata = fs::metadata(path.clone()).unwrap();
    
    // get the last modification time
    let modified = metadata.modified().unwrap();
    
    // get the current system time
    let now = SystemTime::now();
    
    // get the difference between now and modified
    let diff = now.duration_since(modified).unwrap();
    
    // create a duration of 7 days
    let seven_days = Duration::from_secs(7 * 24 * 60 * 60);
    let one_day = Duration::from_secs(1 * 24 * 60 * 60);
    
    // check if diff is less than or equal to seven_days
  //   let date = if diff <= seven_days {
  //     // format modified as a relative date
  //     let modified_date = DateTime::<Utc>::from(modified).with_timezone(&Local);
  //     // let now_date = DateTime::<Utc>::from(now).with_timezone(&Local);
  //     let relative_date = modified_date.format("%R %a").to_string();
  //     println!("{} was modified {}", path, relative_date);
  //     relative_date
  // } else {
  //     // format modified as a UNIX timestamp
  //     let modified_date = DateTime::<Utc>::from(modified);
  //     let unix_timestamp = modified_date.timestamp();
  //     println!("{} was modified at {}", path, unix_timestamp);
  //     unix_timestamp.to_string()
  // };
  let timestamp;
  let modified_date = DateTime::<Utc>::from(modified).with_timezone(&Local);
  timestamp=modified_date.timestamp();
  // timestamp=format!("{}",modified_date.timestamp());
  let now_date = DateTime::<Utc>::from(now).with_timezone(&Local);
  let relative_date = modified_date.format("%R %a").to_string();
  let absolute_date = modified_date.format("%d-%m-%y %H:%S").to_string();
  let date=if diff <= seven_days && diff > one_day {
    // format modified as a relative date
    let diff = now_date.signed_duration_since(modified_date);

    // get the number of days in the difference
    let days = diff.num_days();
    // println!("{} was modified {}", path, days);
    // relative_date
    format!("{} day(s) ago @ {} ",days,relative_date)
    // println!("{} was modified {}", path, relative_date);
    // relative_date
} else if diff <= one_day {
  // format modified as a relative date
  // let modified_date = DateTime::<Utc>::from(modified).with_timezone(&Local);

  // // let now_date = DateTime::<Utc>::from(now).with_timezone(&Local);
  // let relative_date = modified_date.format("%R").to_string();
  
  // println!("{} was modified {}", path, relative_date);
  relative_date
} else{
    // format modified as an absolute date
    // let modified_date = DateTime::<Utc>::from(modified).with_timezone(&Local);
    // println!("{} was modified on {}", path, absolute_date);
    absolute_date
};
    (date,timestamp)
}
fn main() {
  let mut g=FileSizeFinder::new(CACHE_EXPIRY);
  tauri::Builder::default()
    .setup(|app| {
      // get an instance of AppHandle
      // let app_handle = app.handle().get_window("main").unwrap();
      // let g=app.state::<FileSizeFinder>();
    //   // spawn a thread to list the files in the current directory on startup
    // //   std::thread::spawn(move || {
    // //     list_files(".".to_string(), app_handle.get_window("main").unwrap());
    // //   });
    //   // set the window flags to remove WS_MAXIMIZEBOX
    //   app_handle.set_window_flags(|flags| flags & !WS_MAXIMIZEBOX)?;
      Ok(())
    })
    .manage(g)
    .invoke_handler(tauri::generate_handler![
        list_files,
        loadmarkdown,
        get_path_options,
        openpath
        ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
// In Rust, define a function that takes a path as an argument and returns a list of possible paths
#[tauri::command]
fn get_path_options(mut path: String) -> Vec<String> {
  // let whereto=path.clone();
  // let app_handle=window.app_handle();
  // Use some logic to generate a list of possible paths based on the input path
  // For example, use std::fs::read_dir to list the files in a directory
  let mut options = Vec::new();
  let pathasbuf=PathBuf::from(path.clone());
  if(!pathasbuf.exists()){
    if let Some(parent) = pathasbuf.parent() {
      // Convert parent to OsStr
      path = parent.as_os_str().to_string_lossy().to_string();
    }
  }
  // if let Some(index) = path.rfind('/') {
  //   // Get the substring from 0 to index
  //       if let Some(substring) = path.get(0..index+1) {
          // Use substring instead of path
      if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
          if let Ok(entry) = entry {
            // if let Ok(file_name) = entry.path().to_string_lossy().to_string()
            {
              // if(!entry.path().is_file()){

                options.push(entry.path().to_string_lossy().to_string());
              // }
            }
          }
        }
      // }
    // }
  }
  // app_handle.emit_to(
  //   "main",
  //   "pop-datalist",
  //   options.clone(),
  // )
  // .map_err(|e| e.to_string()).unwrap();
  // Return the list of possible paths
  println!("{:?}",options);
  options
}

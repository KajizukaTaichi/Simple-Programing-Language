use std::env;
use std::fs::File;
use std::io::{Error, Read};
mod executor;

#[cfg(test)]
mod tests; //テストモジュールを読み込む

/// ファイルを読み込む
fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

fn main() {
    let message = "Simple プログラミング言語\nコンピュータの動作原理やロジックを学べます\n(c) 2023 梶塚太智. All rights reserved";
    let args = env::args().collect::<Vec<_>>();
    let mut memory = Vec::new();
    let mut name_space = Vec::new();
    let mut executor = executor::Executor::new(&mut memory, &mut name_space, false, true);
    if args.len() >= 3 {
        //ファイルが環境変数にあるか?
        match get_file_contents(args[2].to_string()) {
            Ok(code) => {
                if args[1] == "run" || args[1] == "r" {
                    println!("{message}");
                    executor.script(&code);
                } else if args[1] == "debug" || args[1] == "d" {
                    println!("{}をデバッグします", args[2]);
                    executor.debugger(&code);
                } else if args[1] == "interactive" || args[1] == "i" {
                    println!("{message}");
                    executor.interactive();
                } else {
                    println!("実行モードを正しく指定してください")
                }
            }
            Err(e) => {
                eprintln!("エラー! :{}", e);
            }
        }
    } else if args.len() == 2 {
        if args[1] == "interactive" || args[1] == "i" {
            println!("{message}");
            executor.interactive();
        }
        match get_file_contents(args[1].to_string()) {
            Ok(code) => {
                executor.script(&code);
            }
            Err(e) => {
                eprintln!("エラー! :{}", e);
            }
        }
    } else {
        //ファイルがない場合はインタラクティブで実行する
        println!("{message}");
        executor.interactive();
    }
}

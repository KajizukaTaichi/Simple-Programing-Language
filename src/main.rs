use std::env;
use std::fs::File;
use std::io::{Error, Read};

mod checker; //構文チェッカー
mod executor; //実行処理
mod stdlib; // 標準ライブラリ

#[cfg(test)]
mod tests; //テストモジュールを読み込む

/// ファイルを読み込む
fn get_file_contents(name: String) -> Result<String, Error> {
    let mut f = File::open(name.trim())?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    Ok(contents)
}

/// タイトルを表示する
fn title(msg: String) {
    println!(" OOOOOOO    OO                                  OO             ");
    println!("OO     OO        OO OOOO  OOOOO    OO OOOOOO    OO     OOOOOO  ");
    println!("OO         OOO   OOO    OOO   OO   OOO     OO   OO    OO    OO ");
    println!(" OOOOOOO    OO   OO     OO    OO   OOO     OO   OO    OOOOOOO  ");
    println!("       OO   OO   OO     OO    OO   OO OOOOOO    OO    OO       ");
    println!("OO     OO   OO   OO     OO    OO   OO           OO    OO    OO ");
    println!(" OOOOOOO    OO   OO     OO    OO   OO           OOOO   OOOOOO  \n");
    println!("コンピュータの動作原理を学ぶ新しい教育用プログラミング言語");
    println!("(c) 2023 梶塚太智. All rights reserved\n");

    println!("{msg}");
    println!("--------------------------------------------------------------------");
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let mut memory = Vec::new();
    let mut name_space = Vec::new();
    let mut executor = executor::Executor::new(
        &mut memory,
        &mut name_space,
        executor::ExecutionMode::Script,
    );
    if args.len() >= 3 {
        //ファイルが環境変数にあるか?
        match get_file_contents(args[2].to_string()) {
            Ok(code) => {
                if args[1] == "run" || args[1] == "r" {
                    title(format!("{}を実行します", args[2]));
                    executor.script(&code);
                } else if args[1] == "debug" || args[1] == "d" {
                    title(format!("{}をデバッグします", args[2]));
                    executor.debugger(&code);
                } else if args[1] == "interactive" || args[1] == "i" {
                    executor.interactive();
                } else if args[1] == "check" || args[1] == "c" {
                    title(format!("{}の構文チェックをします", args[2]));
                    executor.check(code.split("\n").map(|x| x.to_string()).collect());
                    println!("完了しました");
                } else {
                    println!("エラー! サブコマンドが不正です")
                }
            }
            Err(e) => {
                eprintln!("エラー! {}", e);
            }
        }
    } else if args.len() == 2 {
        if args[1] == "interactive" || args[1] == "i" {
            title(String::from("対話モードを起動します。"));
            executor.interactive();
        }
        match get_file_contents(args[1].to_string()) {
            Ok(code) => {
                executor.script(&code);
            }
            Err(e) => {
                eprintln!("エラー! {}", e);
            }
        }
    } else {
        //ファイルがない場合はインタラクティブで実行する
        title(String::from("対話モードを起動します。"));
        executor.interactive();
    }
}

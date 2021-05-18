use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

pub trait Interactor {
    fn input(&self, prompt: &str) -> String;
    fn input_letter(&self, prompt: &str, choices: &str) -> String;
    fn select(&self, prompt: &str, choices: &[&str]) -> Option<usize>;
    fn confirm(&self, prompt: &str) -> bool;
    fn info(&self, message: &str);
    fn error(&self, message: &str);
}

pub struct ConsoleInteractor;
impl Interactor for ConsoleInteractor {
    fn input(&self, prompt: &str) -> String {
        Input::new()
            .with_prompt(prompt)
            .allow_empty(false)
            .interact()
            .unwrap()
    }

    fn input_letter(&self, prompt: &str, choices: &str) -> String {
        Input::new()
            .with_prompt(prompt)
            .validate_with(|input: &String| -> Result<(), &str> {
                if choices.contains(input) {
                    Ok(())
                } else {
                    Err("This is not a mail address")
                }
            })
            .interact()
            .unwrap()
    }

    fn select(&self, prompt: &str, choices: &[&str]) -> Option<usize> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .items(choices)
            .interact_opt()
            .unwrap()
    }

    fn info(&self, message: &str) {
        println!("{}", message);
    }

    fn confirm(&self, prompt: &str) -> bool {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .interact()
            .unwrap()
    }

    fn error(&self, message: &str) {
        eprintln!("{}", message);
    }
}


use builtin;
use str;

set edit:completion:arg-completer[fimfic-tracker] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'fimfic-tracker'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'fimfic-tracker'= {
            cand -c 'Extra config file to use'
            cand --config 'Extra config file to use'
            cand --color 'When to use colors'
            cand -v 'Shows verbose output, can be used multiple times to set level of verbosity'
            cand --verbose 'Shows verbose output, can be used multiple times to set level of verbosity'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
            cand -V 'Print version'
            cand --version 'Print version'
            cand track 'Adds stories for tracking and downloads them'
            cand untrack 'Untracks stories'
            cand list 'List all stories that are being tracked'
            cand download 'Checks for updates on tracking list and downloads them if so'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'fimfic-tracker;track'= {
            cand -o 'Overwrites already present stories on cached data'
            cand --overwrite 'Overwrites already present stories on cached data'
            cand -s 'Don''t download stories, only updates cached data'
            cand --skip-download 'Don''t download stories, only updates cached data'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'fimfic-tracker;untrack'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'fimfic-tracker;list'= {
            cand -s 'Show only the ID and title of each tracked story'
            cand --short 'Show only the ID and title of each tracked story'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'fimfic-tracker;download'= {
            cand -f 'Download no matter the presence of updates'
            cand --force 'Download no matter the presence of updates'
            cand -y 'Automatically answers prompts with Y'
            cand --yes 'Automatically answers prompts with Y'
            cand -n 'Automatically answers prompts with N'
            cand --no 'Automatically answers prompts with N'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'fimfic-tracker;help'= {
            cand track 'Adds stories for tracking and downloads them'
            cand untrack 'Untracks stories'
            cand list 'List all stories that are being tracked'
            cand download 'Checks for updates on tracking list and downloads them if so'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'fimfic-tracker;help;track'= {
        }
        &'fimfic-tracker;help;untrack'= {
        }
        &'fimfic-tracker;help;list'= {
        }
        &'fimfic-tracker;help;download'= {
        }
        &'fimfic-tracker;help;help'= {
        }
    ]
    $completions[$command]
}

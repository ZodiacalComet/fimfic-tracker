
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'fimfic-tracker' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'fimfic-tracker'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'fimfic-tracker' {
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Extra config file to use')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Extra config file to use')
            [CompletionResult]::new('--color', 'color', [CompletionResultType]::ParameterName, 'When to use colors')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Shows verbose output, can be used multiple times to set level of verbosity')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Shows verbose output, can be used multiple times to set level of verbosity')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('track', 'track', [CompletionResultType]::ParameterValue, 'Adds stories for tracking and downloads them')
            [CompletionResult]::new('untrack', 'untrack', [CompletionResultType]::ParameterValue, 'Untracks stories')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all stories that are being tracked')
            [CompletionResult]::new('download', 'download', [CompletionResultType]::ParameterValue, 'Checks for updates on tracking list and downloads them if so')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'fimfic-tracker;track' {
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Overwrites already present stories on cached data')
            [CompletionResult]::new('--overwrite', 'overwrite', [CompletionResultType]::ParameterName, 'Overwrites already present stories on cached data')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Don''t download stories, only updates cached data')
            [CompletionResult]::new('--skip-download', 'skip-download', [CompletionResultType]::ParameterName, 'Don''t download stories, only updates cached data')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'fimfic-tracker;untrack' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'fimfic-tracker;list' {
            [CompletionResult]::new('--sort-by', 'sort-by', [CompletionResultType]::ParameterName, 'Sort stories by the given key')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Show only the ID and title of each tracked story')
            [CompletionResult]::new('--short', 'short', [CompletionResultType]::ParameterName, 'Show only the ID and title of each tracked story')
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Reverse the order of the list')
            [CompletionResult]::new('--reverse', 'reverse', [CompletionResultType]::ParameterName, 'Reverse the order of the list')
            [CompletionResult]::new('--show-complete', 'show-complete', [CompletionResultType]::ParameterName, 'Show stories marked as Complete')
            [CompletionResult]::new('--complete', 'complete', [CompletionResultType]::ParameterName, 'Show stories marked as Complete')
            [CompletionResult]::new('--show-incomplete', 'show-incomplete', [CompletionResultType]::ParameterName, 'Show stories marked as Incomplete')
            [CompletionResult]::new('--incomplete', 'incomplete', [CompletionResultType]::ParameterName, 'Show stories marked as Incomplete')
            [CompletionResult]::new('--show-hiatus', 'show-hiatus', [CompletionResultType]::ParameterName, 'Show stories marked as On Hiatus')
            [CompletionResult]::new('--hiatus', 'hiatus', [CompletionResultType]::ParameterName, 'Show stories marked as On Hiatus')
            [CompletionResult]::new('--show-cancelled', 'show-cancelled', [CompletionResultType]::ParameterName, 'Show stories marked as Cancelled')
            [CompletionResult]::new('--cancelled', 'cancelled', [CompletionResultType]::ParameterName, 'Show stories marked as Cancelled')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'fimfic-tracker;download' {
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Download no matter the presence of updates')
            [CompletionResult]::new('--force', 'force', [CompletionResultType]::ParameterName, 'Download no matter the presence of updates')
            [CompletionResult]::new('-y', 'y', [CompletionResultType]::ParameterName, 'Automatically answers prompts with Y')
            [CompletionResult]::new('--yes', 'yes', [CompletionResultType]::ParameterName, 'Automatically answers prompts with Y')
            [CompletionResult]::new('-n', 'n', [CompletionResultType]::ParameterName, 'Automatically answers prompts with N')
            [CompletionResult]::new('--no', 'no', [CompletionResultType]::ParameterName, 'Automatically answers prompts with N')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'fimfic-tracker;help' {
            [CompletionResult]::new('track', 'track', [CompletionResultType]::ParameterValue, 'Adds stories for tracking and downloads them')
            [CompletionResult]::new('untrack', 'untrack', [CompletionResultType]::ParameterValue, 'Untracks stories')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List all stories that are being tracked')
            [CompletionResult]::new('download', 'download', [CompletionResultType]::ParameterValue, 'Checks for updates on tracking list and downloads them if so')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'fimfic-tracker;help;track' {
            break
        }
        'fimfic-tracker;help;untrack' {
            break
        }
        'fimfic-tracker;help;list' {
            break
        }
        'fimfic-tracker;help;download' {
            break
        }
        'fimfic-tracker;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}

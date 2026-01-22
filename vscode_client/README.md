# [Extension Name] - Browser Automation Snippets

**[Extension Name]** provides a comprehensive collection of code snippets to streamline writing browser automation scripts. It focuses on readability and speed, offering shorthand prefixes for common actions like navigation, assertions, looping, and element interaction.

## Features

* **Productivity:** Quickly insert complex automation steps using short, intuitive prefixes.
* **Consistency:** All generated code follows a standardized, lowercase format.
* **Parameter Navigation:** Uses VS Code's tabstops (`$1`, `$2`) to jump between variables (URLs, selectors, data) instantly.

## Usage

Start typing a prefix (e.g., `Maps`, `click`, `if`) and press **Enter** or **Tab** to insert the snippet.

### Example Workflow
Type the following prefixes to generate a complete test flow:

1.  `Maps`
2.  `enter element`
3.  `click element`
4.  `assert`

**Resulting Code:**
```text
navigate [https://example.com](https://example.com)
enter user_login in element #username
click #submit_btn
assert "Welcome Dashboard"
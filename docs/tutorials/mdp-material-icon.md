# How to Create a Custom MDP File Icon (Using Material Icon Theme)

### Prerequisites

- Ensure the **Material Icon Theme** extension is installed in Visual Studio Code.

### Step-by-Step Instructions

1. **Locate and Copy the Source Icon**\
   Copy the `mdx.svg` file from the extension's directory to your custom icon folder:
   - **Source:** `.vscode/extensions/pkief.material-icon-theme-<v>/icons/mdx.svg`
   - **Destination:** `/home/kaizansultan/.vscode/extensions/icons/`

2. **Customize the Icon Color**\
   Open the copied `mdx.svg` file in a text editor and change the `fill` attribute value to `#a78bfa`.

3. **Rename the File**\
   Rename `mdx.svg` to `mdp.svg`.

4. **Configure Visual Studio Code Settings**\
   Add the following association to your `settings.json` file:

   ```json
   "material-icon-theme.files.associations": {
     "*.mdp": "../../icons/mdp"
   }
   ```

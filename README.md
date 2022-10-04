# Roles

Small bot that allows your server members to assign roles to themselves. Without any bloat.

[**Click here to add the bot to your server**](https://discord.com/api/oauth2/authorize?client_id=536761935580889088&permissions=268435456&scope=bot)

Permissions needed: only `Manage Roles`. The bot role also has to be above all selfroles.

# Usage

`/roles`

Creates a select menu with currently available roles to pick from. The menu will keep track of your current selfroles, allowing you to simply uncheck a role to remove it.

`/config add`

Make a role be self-assignable. `label` is the role name displayed in the menu.  
You can update exising selfrole by adding it again.

`/config remove`

Remove a role from the list.

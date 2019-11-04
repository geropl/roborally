/**
 * Implement Gatsby's Node APIs in this file.
 *
 * See: https://www.gatsbyjs.org/docs/node-apis/
 */

// You can delete this file if you're not using it

// TODO Gitpod adjustment: Not needed atm, here for doc purposes
// const { execSync } = require('child_process');
// const { URL } = require('url');

// exports.onCreateWebpackConfig = ({ getConfig, stage, actions }) => {
//     if (stage !== "develop") {
//         return;
//     }
//     const config = getConfig();
//     const output = config.output || {};
//     const url = new URL(execSync('gp url 8000').toString());
//     output.publicPath = 'https://' + url.host + ":443/";
//     actions.setWebpackConfig({
//         output,
//         devServer: {
//             public: url.host + ":443",
//             disableHostCheck: true
//         }
//     });
// }
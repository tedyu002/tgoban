const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: path.resolve(__dirname, "www/bootstrap.js"),
  output: {
    path: path.resolve(__dirname, "../dist"),
    filename: "bootstrap.js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin([path.resolve(__dirname, 'www/index.html')])
  ],
};

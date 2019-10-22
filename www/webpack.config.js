const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
    entry: "./bootstrap.js",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bootstrap.js",
    },
    mode: "development",
    devServer: {
		historyApiFallback: {
			  index: '404.html'
		}
	},
    plugins: [
        new CopyWebpackPlugin(['index.html', '404.html', {
            from: '../src/assets',
            to: 'assets'
        }, {
            from: '../src/style.css',
            to: '.'
        }])
    ],
};

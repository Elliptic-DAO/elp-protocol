module.exports = {
    "env": {
        "browser": true,
        "commonjs": true,
        "es2021": true
    },
    "extends": [
        "eslint:recommended",
        "plugin:react/recommended",
        "plugin:@typescript-eslint/recommended"
    ],
    "overrides": [
    ],
    "parser": "@typescript-eslint/parser",
    "parserOptions": {
        "ecmaVersion": "latest"
    },
    "plugins": [
        "react",
        "@typescript-eslint"
    ],
    "rules": {
    },
    settings: {
        react: {
            version: 'detect'
        }
    },
    ignorePatterns: [
        '/node_modules/',
        '/build/',
        '/dist/',
        'webpack.config.js',
        'canisters/Elliptic_frontend/src/utils/interfaces/'
    ]
}

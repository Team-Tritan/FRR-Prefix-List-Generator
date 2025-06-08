const color = {
    reset: "\x1b[0m",
    green: "\x1b[32m",
    yellow: "\x1b[33m",
    red: "\x1b[31m",
    cyan: "\x1b[36m",
    magenta: "\x1b[35m",
    gray: "\x1b[90m",
};

function log(tag: string, message: string, tagColor = color.cyan) {
    console.log(`${tagColor}[${tag}]${color.reset} ${message}`);
}

function logInfo(tag: string, message: string) {
    log(tag, message, color.green);
}

function logWarn(tag: string, message: string) {
    log(tag, message, color.yellow);
}

function logError(tag: string, message: string) {
    log(tag, message, color.red);
}

function logMagenta(tag: string, message: string) {
    log(tag, message, color.magenta);
}

function logGray(tag: string, message: string) {
    log(tag, message, color.gray);
}

export { log, logInfo, logWarn, logError, logMagenta, logGray, color };

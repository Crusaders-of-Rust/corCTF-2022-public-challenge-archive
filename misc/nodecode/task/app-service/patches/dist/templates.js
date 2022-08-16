"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
const fetch = require("node-fetch");
const { downloadTemplate } = require("../../utilities/fileSystem");
const env = require("../../environment");
// development flag, can be used to test against templates exported locally
const DEFAULT_TEMPLATES_BUCKET = "prod-budi-templates.s3-eu-west-1.amazonaws.com";
exports.fetch = function (ctx) {
    return __awaiter(this, void 0, void 0, function* () {
        ctx.body = [];
    });
};
// can't currently test this, have to ignore from coverage
/* istanbul ignore next */
exports.downloadTemplate = function (ctx) {
    return __awaiter(this, void 0, void 0, function* () {
        const { type, name } = ctx.params;
        yield downloadTemplate(type, name);
        ctx.body = {
            message: `template ${type}:${name} downloaded successfully.`,
        };
    });
};

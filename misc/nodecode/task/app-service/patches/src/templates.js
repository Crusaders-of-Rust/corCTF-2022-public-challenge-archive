const fetch = require("node-fetch")
const { downloadTemplate } = require("../../utilities/fileSystem")
const env = require("../../environment")

// development flag, can be used to test against templates exported locally
const DEFAULT_TEMPLATES_BUCKET =
  "prod-budi-templates.s3-eu-west-1.amazonaws.com"

exports.fetch = async function (ctx) {
  ctx.body = []
}

// can't currently test this, have to ignore from coverage
/* istanbul ignore next */
exports.downloadTemplate = async function (ctx) {
  const { type, name } = ctx.params

  await downloadTemplate(type, name)

  ctx.body = {
    message: `template ${type}:${name} downloaded successfully.`,
  }
}

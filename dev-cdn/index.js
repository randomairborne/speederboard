const express = require("express");
const bodyParser = require('body-parser')
const fs = require("node:fs");

let cdn = express();
function setHeaders(res, _path) {
  res.setHeader("Access-Control-Allow-Origin", "*");
}
cdn.use(express.static("../assets/", { setHeaders }));
cdn.listen(8000, () => {
  console.log("Started server on http://localhost:8000");
});

let fakes3 = express();
fakes3.use(bodyParser.raw({ type: '*' }));
fakes3.use(function (req, res) {
  if (req.method === "PUT") {
    fs.mkdirSync("../assets/" + req.path.substring(0, req.path.lastIndexOf("/")), { recursive: true });
    fs.writeFileSync("../assets/" + req.path, req.body);
  }
  if (req.method === "DELETE") fs.rmSync("../assets/" + req.path);
  res.send("success");
});
fakes3.listen(8001, () => {
  console.log("Started server on http://localhost:8001");
});

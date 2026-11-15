#!/usr/bin/env node
const {spawn}=require('node:child_process');
const {readdirSync}=require('node:fs');
const path=require('node:path');
const root=path.resolve(__dirname,'..');
const target=path.resolve(process.argv[2]||path.join(root,'build/index.html'));
const files=readdirSync(path.join(root,'web')).filter(x=>/^test_.*\.cjs$/.test(x)).sort();
let next=0,failed=0;
async function work(){while(next<files.length){const file=files[next++];const code=await new Promise(resolve=>{
  const child=spawn(process.execPath,[path.join(root,'web',file),target],{cwd:root,stdio:'inherit'});
  child.on('error',error=>{console.error(error);resolve(1)});child.on('exit',code=>resolve(code??1));
});console.log(`${file}: ${code===0?'PASS':'FAIL'}`);if(code!==0)failed++;}}
Promise.all([work(),work()]).then(()=>{console.log(`${files.length-failed}/${files.length} browser suites passed`);process.exitCode=failed?1:0;});

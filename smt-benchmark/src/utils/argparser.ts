const parseArgs = () => {
  const args: Record<string, string> = {};
  process.argv.slice(2).forEach((arg) => {
    if (arg.includes("=")) {
      const [key, value] = arg.split("=");
      args[key] = value;
    }
  });
  // npm run smtbench smt_depth=22 proof_system=groth16
  // => { smt_depth: '22', proof_system: 'groth16' }
  return args;
};

export { parseArgs };

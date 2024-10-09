import { z } from "zod";

export const ProofSystemTypeEnumValidator = z.enum(
  ["groth16", "plonk", "fflonk"],
  {
    errorMap: (issue, ctx) => ({
      message: `Invalid proofSystem. Expected one of groth16, plonk, or fflonk.`,
    }),
  }
);

export type ProofSystemTypeEnum = z.infer<typeof ProofSystemTypeEnumValidator>;

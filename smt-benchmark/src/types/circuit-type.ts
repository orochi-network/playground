import { z } from "zod";

export const CircuitTypeEnumValidator = z.enum(
  ["smt-single-processor", "smt-batch-processor"],
  {
    errorMap: (issue, ctx) => ({
      message:
        'Invalid circuit type. Must be "smt-single-processor" or "smt-batch-processor"',
    }),
  }
);

export type CircuitTypeEnum = z.infer<typeof CircuitTypeEnumValidator>;

export interface CircuitFileGenerator {
  type: CircuitTypeEnum;
  circuitFileGenerator: ({
    tree_depth,
    batch_size,
    path,
  }: {
    tree_depth: number;
    batch_size?: number;
    path?: string;
  }) => void;
}

import { z } from "zod";

export const SMTTaskTypeEnumValidator = z.enum(
  ["insert", "update", "delete", "batch"],
  {
    errorMap: (issue, ctx) => ({
      message: `Invalid SMT task type. Expected one of insert, update, or delete.`,
    }),
  }
);

export type SMTTaskTypeEnum = z.infer<typeof SMTTaskTypeEnumValidator>;

import argparse


def generate(num_fields, num_decls, out):
    template = f"// Generated for {num_decls} of {num_fields} fields\n\n"

    for i in range(num_decls):
        template += f"""
// Some comment for Template{i}
instr_template Template{i}<$param1: bits<100>, $param2: bits<2>, $param3: bits<1>, $param3: str> {{
        """
        for j in range(num_fields):
            template += f"  field{j}: Register,\n"
        template += "}\n\n"

    with open(out, "w") as f:
        f.write(template)

parser = argparse.ArgumentParser()
parser.add_argument("--num-decls", type=int, help="Number of templates to generate")
parser.add_argument("--num-fields", type=int, help="Number of fields in each template")
parser.add_argument("output", help="Output benchmark file")
args = parser.parse_args()

generate(args.num_fields, args.num_decls, args.output)

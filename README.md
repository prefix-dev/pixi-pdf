# Pixi PDF

Pixi PDF is a simple but powerful tool that allows you to embed Pixi projects into a PDF and run them directly from the PDF itself. 

Note: This project is a **proof of concept** to explore the possibility of using PDF as a medium for sharing projects.

## Features

- Embed Pixi projects: Pixi PDF enables you to `embed` your Pixi projects directly into a PDF document. Allowing you to share the project with only the pdf.
- Run projects from PDF: Users can run the embedded Pixi projects directly from the PDF, without the need for any external dependencies or installations. Except for this tool and `pixi` itself.
- Extract a project from PDF: When you want to create a full project from the PDF, you can by extracting the pixi project and run your normal pixi workflow from the toml and lockfile it packs.

## Getting Started

To get started with Pixi PDF, follow these steps:

1. Install [`pixi`](https://pixi.sh)
2. As this is not released, install the project using a pixi task. `pixi run install`
3. Try out the example in `example/`. `pixi-pdf embed -p example -f test.pdf -o test-out.pdf`
4. Check if the `test-out.pdf` is still intact with your favorite pdf viewer.
5. Run the task in the project embedded in the pdf. `pixi-pdf run test-out.pdf -- start` (`start` is the task in the embedded project).
6. Now go and try it on your own projects and have fun!

## Disclaimer

- It uses your system installation of `pixi`, it might be helpfull to get the latest version if you already have one, `pixi self-update`.


## Contributing

This project is a prototype, not being closely maintained. Please feel free to modify it and show us your work. Hopefully this might become an actuall tool we can support. This depends on the community response.

## License

Pixi PDF is released under the [BSD-3 License](./LICENSE). Feel free to use, modify, and distribute this tool for both personal and commercial purposes.

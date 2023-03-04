import { readFileSync } from 'fs';
import { importFromString } from 'module-from-string'
import { Logger } from './logger';
import { isMemoizable, key  } from './library';

import { ts, Project } from "ts-morph";

const logger = Logger.child({
  source: 'Code'
});


export async function transform(source: string) {
  let startTime = performance.now()

  const project = new Project({ useInMemoryFileSystem: true });
  const sourceFile = project.createSourceFile('.//file.ts', source);

  sourceFile.transform(traversal => {
    const node = traversal.visitChildren();

    if (ts.isCallExpression(node)) {
      if (isMemoizable(node.expression.getText())) {
        const k = key();
        const id = traversal.factory.createIdentifier('memoize');
        const typeArgs = undefined;
        const args = [traversal.factory.createStringLiteral(k), node.expression, ...node.arguments];

        return traversal.factory.createCallExpression(id, typeArgs, args);
      }
    }

    return node;
  });

  let endTime = performance.now();
  logger.info(`Transform in ${endTime - startTime}ms`);

  return sourceFile.getFullText();
}


export function sourceFromFile(filename: string) {
  const code = readFileSync(filename, { encoding: 'utf8' });
  return code;
}

export async function codify(source: string): Promise<(() => {})> {
  let filename = 'code.orcvs.js'
  if (['.js', '.orcvs.'].some( s => source.includes(s))) {
    filename = source;
    source = sourceFromFile(source);
  } else {
    logger.info('Loading source as string');
  }

  source = await transform(source);

  const module = `
    export const code = (on) => {
      ${source}
    }
  `

  const { code } = await importFromString(module, { useCurrentGlobal: true, transformOptions: { loader: 'ts' },  filename });
  return code;
}





// function visitor(
//   sourceFile: ts.SourceFile,
//   context: ts.TransformationContext,
//   visitNode: (node: ts.Node, context: ts.TransformationContext) => ts.Node,
// ) {
//   return visitNodeAndChildren(sourceFile) as ts.SourceFile;

//   function visitNodeAndChildren(node: ts.Node): ts.Node {
//     return ts.visitEachChild(visitNode(node, context), visitNodeAndChildren, context);
//   }
// }

// function memoizeLibraryFunctions(node: ts.Node, context: ts.TransformationContext) {
//   if (ts.isCallExpression(node)) {
//     if (isMemoizable(node.expression.getText())) {
//       const k = key();
//       const id = context.factory.createIdentifier('memoize');
//       const typeArgs = undefined;
//       const args = [context.factory.createStringLiteral(k), node.expression, ...node.arguments];

//       return context.factory.createCallExpression(id, typeArgs, args);
//     }
//   }
//   return node;
// }


  // const result = await project.emitToMemory({
  //   customTransformers: {
  //     // optional transformers to evaluate before built in .js transformations
  //     before: [context => sourceFile => visitor(sourceFile, context, memoizeLibraryFunctions)],
  //     // optional transformers to evaluate after built in .js transformations
  //     after: [],
  //     // optional transformers to evaluate after built in .d.ts transformations
  //     afterDeclarations: [],
  //   },
  // });
  // const files = result.getFiles();
  // return files[0].text;
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
    let node = traversal.visitChildren();
    const factory = traversal.factory;

    node = addContextToPulsars(factory, node);
    node = memoizeFunctionCalls(factory, node);

    return node;
  });

  let endTime = performance.now();
  logger.info(`Transform in ${endTime - startTime}ms`);

  return sourceFile.getFullText();
}

function memoizeFunctionCalls(factory: ts.NodeFactory, node: ts.Node) {
  if (ts.isCallExpression(node)) {
    if (ts.isIdentifier(node.expression)) {
      if (isMemoizable(node.expression.getText())) {
        const k = key();
        const id = factory.createIdentifier('memoize');
        const typeArgs = undefined;
        const args = [factory.createStringLiteral(k), node.expression, ...node.arguments];

        return factory.createCallExpression(id, typeArgs, args);
      }

    }
  }
  return node;
}

function addContextToPulsars(factory: ts.NodeFactory, node: ts.Node) {
  if (ts.isCallExpression(node)) {
    if (isPulsar(node)) {
      const propAccess = createPropertyAccessExpression(factory, node);

      let arrow = node.arguments[1] as ts.ArrowFunction;

      if (arrow.parameters.length === 0) {
        const parameters = [factory.createParameterDeclaration(
            undefined,
            undefined,
            factory.createIdentifier('on'),
            undefined,
            undefined,
            undefined
          )];

        const token =  factory.createToken(ts.SyntaxKind.EqualsGreaterThanToken)
        const body = arrow.body;
        arrow = factory.createArrowFunction(undefined, undefined, parameters, undefined, token, body);
      }

      return factory.createCallExpression(
        propAccess,
        undefined,
        [node.arguments[0], arrow]
      )
    }
  }
  return node;
}

function isPulsar(node: ts.CallExpression) {
  const fns = ['ptn', 'at'];
  let text = '';

  if (ts.isPropertyAccessExpression(node.expression)) {
    text = node.expression.name.getText();
  } else {
      text = node.expression.getText();
  }

  return fns.some( s => text === s);
}

function createPropertyAccessExpression(factory: ts.NodeFactory, node: ts.CallExpression) {
  if (ts.isPropertyAccessExpression(node.expression)) {
    return node.expression
  }
  return factory.createPropertyAccessExpression(
    factory.createIdentifier('on'),
    factory.createIdentifier(node.expression.getText())
  )
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

  console.log(code.toString());
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
import { readFileSync } from 'fs';
import { Logger } from './logger';
import { isMemoizable, key  } from './memoize';
import { importFromString } from 'module-from-string'

import { ts, Project } from "ts-morph";
import { OnPulse, Pulsar } from './pulsar';

const logger = Logger.child({
  source: 'Code'
});

export async function transform(source: string) {
  const startTime = performance.now()

  const project = new Project({ useInMemoryFileSystem: true });
  const sourceFile = project.createSourceFile('.//file.ts', source);

  sourceFile.transform(traversal => {
    let node = traversal.visitChildren();
    const factory = traversal.factory;

    node = addContextToPulsars(factory, node);
    node = memoizeFunctionCalls(factory, node);

    return node;
  });

  const endTime = performance.now();
  const elapsed = endTime - startTime;
  logger.info(`Transform in ${elapsed.toFixed(4)}ms`);

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

export async function toCode(filename: string, source: string): Promise<OnPulse> {
  const transformed = await transform(source);

  const module = `
  export const code = (on) => {
    ${transformed}
  }
`
  const { code } = await importFromString(module, { useCurrentGlobal: true, transformOptions: { loader: 'ts' },  filename });

  return code as OnPulse;
}

export function toSource(sourceOrPath: string) {
  let filename = 'code.orcvs.js'
  let source = sourceOrPath;

  if (['.js', '.orcvs.'].some( s => sourceOrPath.includes(s))) {
    filename = sourceOrPath;
    source = sourceFromFile(sourceOrPath);
  } else {
    logger.info('Loading source as string');
  }

  return { filename, source };
}

export interface Runnable {
  run: (pulsar: Pulsar, force?: boolean) => void;
  pending: boolean
}

export async function Code(sourceOrPath: string): Promise<Runnable> {
  const { filename, source } = toSource(sourceOrPath);

  const _code = await toCode(filename, source);

  let _pending = true;

  function run(pulsar: Pulsar, force = false) {
    if ( _pending || force) {
      _pending = false;
      return _code(pulsar);
    }
  }

  return {
    run,
    get pending() {
      return _pending;
    }
  }

}
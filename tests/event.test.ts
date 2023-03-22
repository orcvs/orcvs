import {jest} from '@jest/globals'

import { send, listen, clear } from '../src/event';

require('../src/globals');

describe('event', () => {

  test('send calls listener', async () => {

    const mock = jest.fn();

    listen('test', mock);
    send('test');

    expect(mock).toBeCalled();
  });


  test('ckear all listeners', async () => {

    const mock = jest.fn();

    listen('test', mock);
    clear();
    send('test');

    expect(mock).not.toBeCalled();
  });

});



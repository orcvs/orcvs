import {jest} from '@jest/globals'

import { Eventer } from '../src/eventer';

require('../src/globals');

describe('event', () => {

  test('listener is called', async () => {
    const event = Eventer();

    const mock = jest.fn();

    event.listen('test', mock);
    event.send('test');

    event.tick();

    expect(mock).toBeCalled();
  });

  test('listener is only called once', async () => {
    const event = Eventer();

    const mock = jest.fn();

    event.listen('test', mock);
    event.send('test');

    event.tick();
    expect(mock).toBeCalled();

    event.send('test');

    expect(mock).toBeCalledTimes(1);
  });

  test('clear all listeners', async () => {
    const mock = jest.fn();

    const event = Eventer();

    event.listen('test', mock);
    event.send('test');

    event.clear();

    event.tick();

    expect(mock).not.toBeCalled();
  });

  test('clear listeners for an event', async () => {
    const mock = jest.fn();
    const mockII = jest.fn();

    const event = Eventer();

    event.listen('test', mock);
    event.listen('testII', mockII);
    event.send('test');
    event.send('testII');

    event.clear('test');

    event.tick();

    expect(mock).not.toBeCalled();
    expect(mockII).toBeCalled();
  });

});



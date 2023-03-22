import {jest} from '@jest/globals'

import { Event } from '../src/event';

require('../src/globals');

describe('event', () => {

  test('send calls listener', async () => {
    const event = Event();

    const mock = jest.fn();

    event.listen('test', mock);
    event.send('test');

    event.tick();

    expect(mock).toBeCalled();
  });


  test('clear all listeners', async () => {
    const mock = jest.fn();

    const event = Event();

    event.listen('test', mock);
    event.send('test');

    event.clear();

    event.tick();

    expect(mock).not.toBeCalled();
  });

  test('clear listeners for an event', async () => {
    const mock = jest.fn();
    const mockII = jest.fn();

    const event = Event();

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



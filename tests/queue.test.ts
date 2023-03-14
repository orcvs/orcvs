import {jest} from '@jest/globals'

import { Queue } from '../src/queue';


describe('Queue', () => {

    test('has constant length', async () => {
      const q = Queue(5);

      for (let i = 0; i < 10; i++) {
        q.push(i);
      }

      expect(q.length).toEqual(5);
      expect(q.peak()).toEqual(5);
    });

    test('average', async () => {
      const q = Queue(5);

      for (let i = 1; i <= 5; i++) {
        q.push(i);
      }

      // 1 2 3 4 5
      expect(q.average()).toEqual('3.0000');
    });


    test('90th percentile', async () => {
      const q = Queue();

      for (let i = 1; i <= 100; i++) {
        q.push(i);
      }

      expect(q.percentile()).toEqual('91.0000');
    });
});



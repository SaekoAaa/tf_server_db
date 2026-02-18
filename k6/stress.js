
// LOAD TESTING
import http from 'k6/http';

export const options = {
  scenarios: {
    constant_load: {
      executor: 'constant-arrival-rate',
      rate: 10000,           // 100 запросов
      timeUnit: '1s',      // в секунду
      duration: '30s',     // 30 секунд
      preAllocatedVUs: 20, // заранее созданные VU
      maxVUs: 100,
    },
  },
};

export default function () {
  http.get('http://server:80/todos');
}

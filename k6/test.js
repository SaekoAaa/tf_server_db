// STRESS TEST
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 5,        // 1 виртуальный пользователь
  iterations: 100, // всего 100 запросов
};

export default function () {
  const res = http.get('http://server:80');

  check(res, {
    'status is 200': (r) => r.status === 200,
  });

  sleep(0.1); // небольшая пауза между запросами (100ms)
}

import http from 'k6/http';

export const options = {
  scenarios: {
    ramped_rps: {
      executor: 'ramping-arrival-rate',
      startRate: 10000,       
      timeUnit: '1s',
      preAllocatedVUs: 500,  // начальный пул
      maxVUs: 20000,
      stages: [
        { target: 10000, duration: '20s' }, // 0–20с → 10k RPS
        { target: 20000, duration: '20s' }, // 20–40с → 20k RPS
        { target: 30000, duration: '20s' }, // 40–60с → 30k RPS
      ],
    },
  },
};

export default function () {
  http.get('http://server:80/');
}

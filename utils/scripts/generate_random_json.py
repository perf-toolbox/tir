import json
import random
from faker import Faker
import sys

data = []

fake = Faker(sys.argv[1])

for _ in range(int(sys.argv[2])):
    data.append({
        'client_id': random.randint(1000000, 10000000),
        'name': fake.name(),
        'email': fake.email(),
        'age': random.randint(16, 99),
        'address': {
            'street': fake.street_address(),
            'city': fake.city(),
            'zip': fake.zipcode(),
        }
    })

json_data = json.dumps(data, indent=4)
print(json_data)

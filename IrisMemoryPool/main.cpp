#include "IrisObjectMemoryPoolInterface.h"
#include "IrisAbstractMemoryPool.h"

#include <Windows.h>
#include <iostream>
using namespace std;

class TestClass : public IrisObjectMemoryPoolInterface<TestClass> {
private:
	int m_nInterger = 0;
	float m_fFloat = 0.0f;
	double m_dDouble = 0.0;
	char m_cChar = 'a';

public:
	TestClass() {}
	~TestClass() {}
};

class TestClass2 {
private:
	int m_nInterger = 0;
	float m_fFloat = 0.0f;
	double m_dDouble = 0.0;
	char m_cChar = 'a';

public:
	TestClass2() {}
	~TestClass2() {}
};

const int nTime = 10000000;

int main(int argc, char* argv[]) {
	TestClass* pObject = nullptr;
	
	auto t1 = ::GetTickCount64();
	list<TestClass*> lsT1;
	for (int i = 0; i < nTime; ++i) {
		pObject = new TestClass();
		lsT1.push_back(pObject);
	}
	for (auto p : lsT1) {
		delete p;
	}
	cout << "time 1 : " << ::GetTickCount64() - t1 << endl;

	t1 = ::GetTickCount64();
	list<TestClass2*> lsT2;
	TestClass2* pObject2 = nullptr;
	for (int i = 0; i < nTime; ++i) {
		pObject2 = new TestClass2();
		lsT2.push_back(pObject2);
	}
	for (auto p : lsT2) {
		delete p;
	}
	cout << "time 2 : " << ::GetTickCount64() - t1 << endl;

	IrisAbstractMemoryPool::ReleaseAllMemoryPool();

	return 0;
}
#ifndef _H_IRISVALUES_
#define _H_IRISVALUES_

#include <IrisUnil/IrisValue.h>
#include <IrisInterfaces/IIrisValues.h>

#include <functional>
#include <initializer_list>
#include <vector>
using namespace std;

class IrisValues : public IIrisValues
{
private:
	vector<IrisValue> m_vcValues;

public:
	IrisValues(const initializer_list<IrisValue>& ilValuesList);
	IrisValues();
	IrisValues(size_t nSize);

	void Add(const IrisValue& ivValue);

	inline vector<IrisValue>& GetVector() { return m_vcValues; }

	inline const IrisValue& GetValue(size_t nIndex) const
	{
		return m_vcValues[nIndex];
	}

	//bool Ergodic(const function<bool(IrisValue&)>& fErgodicFunction);

	inline IrisValue& operator [](size_t nIndex) {
		return (IrisValue&)GetValue(nIndex);
	}

	inline size_t GetSize() {
		return m_vcValues.size();
	}

	~IrisValues();
};

#endif
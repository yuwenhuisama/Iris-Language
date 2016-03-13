#ifndef _H_IRISHASHTAG_
#define _H_IRISHASHTAG_

#include "IrisDevHeader.h"
#include "IrisInteger.h"
#include "IrisFloat.h"
#include "IrisString.h"

#include <unordered_map>
#include <math.h>  
using namespace std;

#ifdef IR_USE_MEM_POOL
class IrisHashTag : public IrisObjectMemoryPoolInterface<IrisHashTag, POOLID_IrisHashTag>
#else
class IrisHashTag
#endif
{
private:
	struct IrisValueHash {
		size_t operator () (const IrisValue& ivValue) const {

			if (((IrisValue&)ivValue).GetIrisObject()->Hashed()) {
				return ((IrisValue&)ivValue).GetIrisObject()->GetHash();
			}

			size_t nHash = 0;
			if (IrisDevUtil::CheckClass((IrisValue&)ivValue ,"Integer")) {
				nHash = size_t(IrisDevUtil::GetInt((IrisValue&)ivValue));
			}
			else if (IrisDevUtil::CheckClass((IrisValue&)ivValue, "Float")) {
				int e = 0;
				double dValue = IrisDevUtil::GetFloat((IrisValue&)ivValue);
				double tmp = dValue;
				if (dValue<0)
				{
					tmp = -dValue;
				}
				e = (int)ceil(log(dValue));
				nHash = size_t((INT64_MAX + 1.0) * tmp * exp(-e));
			}
			else if (IrisDevUtil::CheckClass((IrisValue&)ivValue, "String")) {
				const string& str = IrisString::GetString((IrisValue&)ivValue);
				size_t h = 0;
				for (size_t i = 0;i < str.length();++i)
				{
					h = (h << 5) - h + str[i];
				}
				nHash = h;
			}
			else {
				nHash = (size_t)((IrisValue&)ivValue).GetIrisObject()->GetObjectID();
			}
			((IrisValue&)ivValue).GetIrisObject()->SetHash(nHash);
			return nHash;
		}
	};

	struct IrisValueCmp {
		bool operator()(const IrisValue& ivValue1, const IrisValue& ivValue2) const {
			IrisValues ivValues = { ivValue2 };
			return IrisDevUtil::CallMethod((IrisValue&)ivValue1, &ivValues, "==") == IrisDevUtil::True();
		}
	};

public:
	typedef unordered_map<IrisValue, IrisValue, IrisValueHash, IrisValueCmp> _IrisHash;

private:
	_IrisHash m_mpHash;

public:

	void Initialize(IrisValues* pValues);
	IrisValue At(const IrisValue& ivIndex);
	IrisValue Set(const IrisValue& ivKey, const IrisValue& ivValue);
	int Size();

	IrisHashTag();
	~IrisHashTag();

	friend class IrisHash;
	friend class IrisHashIterator;
};

#endif
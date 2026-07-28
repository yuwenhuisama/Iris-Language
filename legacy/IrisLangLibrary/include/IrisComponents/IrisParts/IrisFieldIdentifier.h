#ifndef _H_IRISFIELDIDENTIFIER_
#define _H_IRISFIELDIDENTIFIER_

#include "IrisUnil/IrisList.h"

class IrisIdentifier;
class IrisFieldIdentifier
{
public:
	IrisList<IrisIdentifier*>* m_pList = nullptr;
	IrisIdentifier* m_pIdentifier = nullptr;
	bool m_bIsTopField = false;

public:
	IrisFieldIdentifier(IrisList<IrisIdentifier*>* pList, IrisIdentifier* pIdentifier, bool bIsTopField);
	~IrisFieldIdentifier();
};

#endif
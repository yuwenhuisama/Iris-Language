#ifndef _H_IRISSATAEMENT_
#define _H_IRISSATAEMENT_
#include "IrisStatement.h"

class IrisIdentifier;
class IrisBreakStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pLabel = nullptr;

public:

	virtual bool Generate();

	IrisBreakStatement(IrisIdentifier* pLabel);
	virtual ~IrisBreakStatement();

	virtual bool Validate() override;
};

#endif